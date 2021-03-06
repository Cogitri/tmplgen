//This file is part of tmplgen
//
//tmplgen is free software: you can redistribute it and/or modify
//it under the terms of the GNU General Public License as published by
//the Free Software Foundation, either version 3 of the License, or
//(at your option) any later version.
//
//tmplgen is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU General Public License for more details.
//
//You should have received a copy of the GNU General Public License
//along with tmplgen.  If not, see <http://www.gnu.org/licenses/>.

use crate::errors::Error;
use crate::helpers::*;
use crate::types::*;
use log::{debug, info, warn};
use rayon::prelude::*;
use std::path::Path;

impl TmplBuilder {
    /// Initializes a new TmplBuilder with nothing but pkg_name set.
    pub fn new(pkg_name: &str) -> Self {
        Self {
            pkg_name: pkg_name.to_owned(),
            pkg_type: None,
            pkg_info: None,
        }
    }

    /// Initializes a new TmplBuilder from a PkgInfo. Useful for testing or as a shortcut
    pub fn from_pkg_info(pkg_info: PkgInfo) -> Self {
        Self {
            pkg_name: pkg_info.pkg_name.clone(),
            pkg_type: None,
            pkg_info: Some(pkg_info),
        }
    }

    /// Gets the PkgType of the package of the TmplBuilder that's passed into the method
    ///
    /// # Errors
    ///
    /// * If a package with the name of (self.pkg_name)[crate::TmplBuilder.pkg_name] can be
    ///   found on multiple platforms (e.g. on both (crates.io)[https://crates.io] and (rubygems.org)[https://rubygems.org])
    /// * If the package can't be found on any of the platforms
    pub fn get_type(&mut self) -> Result<&mut Self, Error> {
        self.pkg_type = Some(figure_out_provider(&self.pkg_name)?);
        Ok(self)
    }

    /// Sets the PkgType of the package of the TmplBuilder that's passed into the method
    pub fn set_type(&mut self, pkg_type: PkgType) -> &mut Self {
        self.pkg_type = Some(pkg_type);
        self
    }

    /// Gets the PkgInfo of the package of the TmplBuilder that's passed into the method
    ///
    /// # Errors
    ///
    /// * If you try to call this method without setting/getting pkg_type first via either
    ///   (self.get_type)[crate::tmplwriter::TmplBuilder::get_type] or (self.set_type)[crate::tmplwriter::TmplBuilder::set_type]
    pub fn get_info(&mut self) -> Result<&mut Self, Error> {
        if self.pkg_type.is_some() {
            self.pkg_info = Some(get_pkginfo(&self.pkg_name, self.pkg_type.unwrap())?);
            Ok(self)
        } else {
            Err(Error::TooLittleInfo(
                "Can't get PkgInfo without setting/getting PkgType first!".to_string(),
            ))
        }
    }

    /// Sets the PkgInfo of the package of the TmplBuilder that's passed into the method
    pub fn set_info(&mut self, pkg_info: PkgInfo) -> &mut Self {
        self.pkg_info = Some(pkg_info);
        self
    }

    /// Checks if a Gem or PerlDist is built into Ruby/Perl.
    ///
    /// # Errors
    ///
    /// * If you try to call this method without setting/getting pkg_type first via either
    ///   (self.get_type)[crate::tmplwriter::TmplBuilder::get_type] or (self.set_type)[crate::tmplwriter::TmplBuilder::set_type]
    pub fn is_built_in(&self) -> Result<bool, Error> {
        if self.pkg_type.is_some() {
            let data: TomlData = toml::from_str(include_str!("data.toml")).unwrap();

            let built_ins = BuiltInDeps {
                perl: data.builtin.perl,
                ruby: data.builtin.ruby,
            };

            if self.pkg_type.unwrap() == PkgType::Gem {
                for x in built_ins.ruby {
                    if self.pkg_name == x {
                        info!(
                            "Gem {} is part of ruby, won't write a template for it!",
                            self.pkg_name
                        );
                        return Ok(true);
                    }
                }
            } else if self.pkg_type.unwrap() == PkgType::PerlDist {
                let pkg_name = self.pkg_name.replace("::", "-");

                for x in built_ins.perl {
                    if pkg_name == x {
                        info!(
                            "Perl distribution {} is part of perl, won't write a template for it!",
                            pkg_name
                        );
                        return Ok(true);
                    }
                }
            }

            // Crates can't be built in
            Ok(false)
        } else {
            Err(Error::TooLittleInfo(
                "Can't check if Pkg is built in without setting/getting PkgType first!".to_string(),
            ))
        }
    }

    /// Helper method to get a Vec<[Template](crate::types::Template)> of all dependencies a
    /// package has. Also includes recursive dependencies.
    ///
    /// # Arguments
    ///
    /// Takes the optional argument 'tmpl_path', which is used to check if a template exists already
    /// and as such not unnecessarily return a Template struct for a template we don't need (because
    /// it exists already)
    ///
    /// # Errors:
    ///
    /// * If you try to call this method without actually getting the deps via [get_deps](crate::TmplBuilder::get_deps) first
    /// * If something went wrong while generating Templates for all dependencies of [self.pkg_name](crate::TmplBuilder.pkg_name)
    // TODO: Make this prettier so we don't needlessly generate templates twice if dep x and y both depend on z
    pub fn gen_deps(&self, tmpl_path: Option<&str>) -> Result<Vec<Template>, Error> {
        if self.pkg_info.is_some() {
            let deps = self
                .pkg_info
                .as_ref()
                .unwrap()
                .dependencies
                .as_ref()
                .unwrap();

            let mut run_vec = if let Some(run_deps) = &deps.run {
                run_deps
                    .par_iter()
                    .filter(|x| x == &&"perl".to_string() || x == &&"ruby".to_string())
                    .collect::<Vec<&String>>()
            } else {
                Vec::new()
            };

            let mut make_vec = if let Some(make_deps) = &deps.make {
                make_deps
                    .par_iter()
                    .filter(|x| x == &&"perl".to_string() || x == &&"ruby".to_string())
                    .collect::<Vec<&String>>()
            } else {
                Vec::new()
            };

            run_vec.append(&mut make_vec);

            let mut tmpl_vec = Vec::new();

            for x in run_vec {
                let pkg_tainted = x.split('>').collect::<Vec<&str>>()[0]
                    .split('<')
                    .collect::<Vec<&str>>()[0];

                let pkg = pkg_tainted.replace("perl-", "").replace("ruby-", "");

                let mut tmpl_builder = Self::new(&pkg);

                if tmpl_builder
                    .set_type(self.pkg_type.unwrap())
                    .is_built_in()?
                {
                    debug!("Won't write template for built-in package {}", pkg);
                    continue;
                } else if tmpl_path.is_some() {
                    let tmpl_path_exists = if self.pkg_type.unwrap() == PkgType::Gem {
                        Path::new(&format!(
                            "{}/ruby-{}/template",
                            tmpl_path.unwrap_or_default(),
                            pkg
                        ))
                        .exists()
                    } else {
                        Path::new(&format!(
                            "{}/perl-{}/template",
                            tmpl_path.unwrap_or_default(),
                            pkg.replace("::", "-")
                        ))
                        .exists()
                    };

                    if tmpl_path_exists {
                        continue;
                    }
                }

                tmpl_vec.push(tmpl_builder.get_info()?.generate(true)?);

                tmpl_vec.append(
                    &mut Self::new(&pkg)
                        .set_type(self.pkg_type.unwrap())
                        .get_info()?
                        .gen_deps(tmpl_path)?,
                )
            }
            Ok(tmpl_vec)
        } else {
            Err(Error::TooLittleInfo(
                "Can't create Templates for deps without setting/getting PkgInfo of the package first!".to_string(),
            ))
        }
    }

    /// Updates a [Template](crate::types::Template)
    ///
    /// # Example
    /// ```
    /// use libtmplgen::*;
    /// use std::fs::File;
    /// use std::io::prelude::*;
    ///
    /// fn update_template() -> Result<(), Error> {
    ///     // Do note that we only manually create a PkgInfo here to make the example easier to understand
    ///     let pkg_info_crate = PkgInfo {
    ///            pkg_name: "tmplgen".to_string(),
    ///            version: "0.6.0".to_string(),
    ///            description: Some("Void Linux template generator for language-specific package managers"
    ///                .to_string()),
    ///            homepage: "https://github.com/Cogitri/tmplgen".to_string(),
    ///            license: Some(vec!["GPL-3.0-or-later".to_string()]),
    ///            dependencies: None,
    ///            sha: "afc403bf69ad4da168938961b0f02da86ef29d655967cfcbacc8201e1327aff4".to_string(),
    ///            download_url: Some(
    ///               "https://static.crates.io/crates/tmplgen/tmplgen-${version}.crate".to_string(),
    ///            ),
    ///     };
    ///
    ///     let mut old_template = Template { inner: String::new(), name: "tmplgen".to_string() };
    ///     // Open whatever file you want to below.
    ///     let mut file = File::open("src/lib/tests/template_test_crate.in")?;
    ///     file.read_to_string(&mut old_template.inner)?;
    ///
    ///     // This will return a [Template](crate::types::Template), which is updated to 0.6.0 (as specified by pkg_info_crate above)
    ///     // Use TmplBuilder::new("tmplgen").get_info() to manually get the info we manually set in pkg_info_crate!
    ///     // Won't update `homepage`, `distfiles` and `short_desc`, set the second argument to `true` for that
    ///     let template_updated = TmplBuilder::from_pkg_info(pkg_info_crate)
    ///         .get_type()?
    ///         .update(&old_template, false)?;
    ///
    ///     // Write the [Template](crate::types::Template) to `./template`
    ///     let mut file = File::create("./template")?;
    ///     file.write_all(template_updated.inner.as_bytes())?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// * If you try to call this method without setting/getting pkg_info first via either
    ///   (self.get_info)[crate::tmplwriter::TmplBuilder::get_info] or (self.set_type)[crate::tmplwriter::TmplBuilder::set_info]
    pub fn update(&self, old_template: &Template, update_all: bool) -> Result<Template, Error> {
        let pkg_info = if self.pkg_info.is_some() {
            Ok(self.pkg_info.as_ref().unwrap())
        } else {
            Err(Error::TooLittleInfo(
                "Can't update template without setting PkgInfo first!".to_string(),
            ))
        }?;

        info!("Updating template {}", &pkg_info.pkg_name);

        let mut template_string = old_template.inner.clone();

        let mut orig_vec = vec![
            "version=",
            "checksum=",
            "distfiles=",
            "homepage=",
            "short_desc=",
            "revision=",
        ]
        .par_iter()
        .map(|search_str| {
            template_string
                .lines()
                .filter(|x| x.contains(search_str))
                .map(|x| x)
                .collect::<String>()
        })
        .collect::<Vec<String>>();

        // Reversed order of orig_vec, we pop off the end
        let orig_revision_string = orig_vec.pop().unwrap_or_else(|| "".to_string());
        let orig_description_string = orig_vec.pop().unwrap_or_else(|| "".to_string());
        let orig_homepage_string = orig_vec.pop().unwrap_or_else(|| "".to_string());
        let orig_distfiles_string = orig_vec.pop().unwrap_or_else(|| "".to_string());
        let orig_checksum_string = orig_vec.pop().unwrap_or_else(|| "".to_string());
        let orig_ver_string = orig_vec.pop().unwrap_or_else(|| "".to_string());

        if !(orig_ver_string == format!("version={}", &pkg_info.version)) {
            template_string = template_string.replace(&orig_revision_string, "revision=1");
        }
        template_string =
            template_string.replace(&orig_ver_string, &format!("version={}", &pkg_info.version));

        if update_all {
            template_string = template_string.replace(
                &orig_checksum_string,
                &format!("checksum={}", &pkg_info.sha),
            );

            if orig_homepage_string.is_empty() {
                warn!("Couldn't find 'homepage' string and as such won't update it!");
            } else {
                template_string = template_string.replace(
                    &orig_homepage_string,
                    &format!("homepage=\"{}\"", &pkg_info.homepage),
                );
            }

            if orig_distfiles_string.is_empty() {
                warn!("Couldn't find 'distfiles' string and as such won't update it!");
            } else {
                // This looks a bit funny because...well, it is. download_url can be empty, but we
                // want to just remove the previous distfiles, in case the gem downloads some additional
                // data
                template_string = template_string.replace(
                    &orig_distfiles_string,
                    &format!(
                        "distfiles=\"{}",
                        format!(
                            "{}\"",
                            &pkg_info
                                .download_url
                                .as_ref()
                                .unwrap_or(&orig_distfiles_string.replace("distfiles=\"", ""))
                        )
                    ),
                );
            }

            if orig_description_string.is_empty() {
                warn!("Couldn't find 'description' string and as such won't update it!");
            } else if pkg_info.description.is_some() {
                template_string = template_string.replace(
                    &orig_description_string,
                    &format!("short_desc=\"{}\"", pkg_info.description.as_ref().unwrap()),
                );
            } else {
                warn!("Couldn't determine field 'description'! Won't update it.",);
            }
        } else {
            // If we don't update all (and as such also update distfiles) we want to download the
            // file specified in distfiles and write its checksum to the template
            let tmpl_download_url = &orig_distfiles_string
                .replace("distfiles=", "")
                .replace("\"", "");

            // If the download url we determined matches the one we pulled from the template
            // we can just use the sha we already know
            if pkg_info.download_url.as_ref().unwrap_or(&"".to_string()) == tmpl_download_url {
                template_string = template_string.replace(
                    &orig_checksum_string,
                    &format!("checksum={}", &pkg_info.sha),
                );
            } else {
                // If it doesn't match we have to download the distfile and get its sha sum
                template_string = template_string.replace(
                    &orig_checksum_string,
                    &format!(
                        "checksum={}",
                        gen_checksum(&tmpl_download_url.replace("${version}", &pkg_info.version))?
                    ),
                );
            };
        }

        Ok(Template {
            inner: template_string.to_owned(),
            name: pkg_info.pkg_name.clone(),
        })
    }

    /// Generates a new [Template](crate::types::Template)
    ///
    /// # Example
    ///
    /// ```
    /// use libtmplgen::*;
    /// use std::fs::File;
    /// use std::io::prelude::*;
    ///
    /// fn write_template() -> Result<(), Error> {
    ///     // Do note that we only manually create a PkgInfo here to make the example easier to understand
    ///     let pkg_info_crate = PkgInfo {
    ///            pkg_name: "tmplgen".to_string(),
    ///            version: "0.6.0".to_string(),
    ///            description: Some("Void Linux template generator for language-specific package managers"
    ///                .to_string()),
    ///            homepage: "https://github.com/Cogitri/tmplgen".to_string(),
    ///            license: Some(vec!["GPL-3.0-or-later".to_string()]),
    ///            dependencies: None,
    ///            sha: "afc403bf69ad4da168938961b0f02da86ef29d655967cfcbacc8201e1327aff4".to_string(),
    ///            download_url: Some(
    ///               "https://static.crates.io/crates/tmplgen/tmplgen-${version}.crate".to_string(),
    ///            ),
    ///     };
    ///
    ///     // Use TmplBuilder::new("tmplgen").get_type.generate() to do this automatically instead of
    ///     // setting PkgInfo and PkgType manually
    ///     // You generally want "prefix" to be true, unless you want to generate a template without
    ///     // the language- prefix.
    ///     let template = TmplBuilder::from_pkg_info(pkg_info_crate).set_type(PkgType::Crate).generate(true)?;
    ///
    ///     // Write the [Template](crate::types::Template) to `./template`
    ///     let mut file = File::create("./template")?;
    ///     file.write_all(template.inner.as_bytes())?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// * If you try to call this method without setting/getting pkg_info first via either
    ///   (self.get_info)[crate::tmplwriter::TmplBuilder::get_info] or (self.set_info)[crate::tmplwriter::TmplBuilder::set_info]
    pub fn generate(&self, prefix: bool) -> Result<Template, Error> {
        let pkg_info = if self.pkg_info.is_some() {
            Ok(self.pkg_info.as_ref().unwrap())
        } else {
            Err(Error::TooLittleInfo(
                "Can't write a new template without setting PkgInfo first!".to_string(),
            ))
        }?;

        // Can't have pkg_info without pkg_type, so we only have to check for pkg_info's existance
        let tmpl_type = self.pkg_type.unwrap();

        let template_in = include_str!("template.in");

        let maintainer = get_git_author()?;

        let mut template_string = template_in
            .replace("@version@", &pkg_info.version)
            .replace("@maintainer@", &maintainer)
            .replace("@pkgname@", &pkg_info.pkg_name)
            .replace("@checksum@", &pkg_info.sha)
            .replace("@homepage@", &pkg_info.homepage);

        if pkg_info.description.is_some() {
            let mut description = check_string_len(
                &pkg_info.pkg_name,
                &pkg_info.description.as_ref().unwrap(),
                "description",
            );

            if description.chars().last().unwrap_or_default() == '.' {
                description.pop();
            }

            template_string = template_string.replace("@description@", &description)
        } else {
            warn!(
                "Couldn't determine field 'description'! Please add it to the template yourself.",
            );
        }

        if pkg_info.license.is_some() {
            let license_iter = pkg_info.license.as_ref().unwrap().par_iter();
            let mut license = license_iter
                .clone()
                .take(1)
                .map(|x| correct_license(x))
                .collect::<String>();

            license.push_str(
                &license_iter
                    .skip(1)
                    .map(|x| format!(", {}", correct_license(x)))
                    .collect::<String>(),
            );

            template_string = template_string.replace("@license@", &license)
        } else {
            warn!("Couldn't determine field 'license'! Please add it to the template yourself.",);
        }

        if pkg_info.dependencies.is_some() {
            let dependencies = pkg_info.dependencies.as_ref().unwrap();

            if dependencies.host.is_some() {
                let host_depends = gen_dep_string(dependencies.host.as_ref().unwrap(), tmpl_type);
                template_string =
                    template_string.replace("@hostmakedepends@", &host_depends.trim_end());
            } else {
                template_string =
                    template_string.replace("\nhostmakedepends=\"@hostmakedepends@\"", "");
            }
            if dependencies.make.is_some() {
                let make_depends = gen_dep_string(dependencies.make.as_ref().unwrap(), tmpl_type);
                template_string =
                    template_string.replace("@makedepends@", &make_depends.trim_end());
            } else {
                template_string = template_string.replace("\nmakedepends=\"@makedepends@\"", "");
            }
            if dependencies.run.is_some() {
                let run_depends = gen_dep_string(dependencies.run.as_ref().unwrap(), tmpl_type);
                template_string = template_string.replace("@depends@", &run_depends.trim_end());
            } else {
                template_string = template_string.replace("\ndepends=\"@depends@\"", "");
            }
        } else {
            template_string = template_string.replace("\ndepends=\"@depends@\"", "");
            template_string = template_string.replace("\nmakedepends=\"@makedepends@\"", "");
            template_string =
                template_string.replace("\nhostmakedepends=\"@hostmakedepends@\"", "");
        }

        if pkg_info.download_url.is_some() {
            template_string =
                template_string.replace("@distfiles@", &pkg_info.download_url.as_ref().unwrap());
        } else {
            template_string = template_string.replace("\ndistfiles=\"@distfiles@\"", "")
        }

        if tmpl_type == PkgType::PerlDist {
            template_string = template_string
                .replace("@build_style@", "perl-module")
                .replace("@archs@", "noarch");
        } else if tmpl_type == PkgType::Gem {
            template_string = template_string
                .replace("@build_style@", "gem")
                .replace("\nwrksrc=\"@wrksrc@\"", "")
                .replace("\narchs=@archs@", "");
        } else {
            template_string = template_string
                .replace("@pkgname@", &pkg_info.pkg_name)
                .replace("\ndepends=\"@depends@\"", "")
                .replace("@build_style@", "cargo")
                .replace("\narchs=@archs@", "");
        }

        if prefix {
            let prefix_str = if tmpl_type == PkgType::Crate {
                "rust-"
            //} else if tmpl_type == PkgType::Gem {
            //    "ruby-"
            } else {
                "perl-"
            };

            let wrksrc = format!("${{pkgname/{}/}}-${{version}}", prefix_str);

            template_string = template_string.replace("@wrksrc@", &wrksrc);
        } else {
            template_string = template_string.replace("\nwrksrc=\"@wrksrc@\"", "");
        }

        let license = &pkg_info.license.as_ref().unwrap_or(&Vec::new()).join(", ");
        if license.contains(&"MIT".to_string())
            || license.contains(&"ISC".to_string())
            || license.contains(&"BSD".to_string())
        {
            template_string.push_str("\n\npost_install() {\n\tvlicense LICENSE\n}");
        }

        template_string.push_str("\n");

        Ok(Template {
            inner: template_string,
            name: pkg_info.pkg_name.clone(),
        })
    }
}
