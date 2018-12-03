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
use crate::perldist::perldist_dep_graph;
use crate::gems::gem_dep_graph;

use log::{info, warn};

impl TmplBuilder {
    /// Initializes a new TmplBuilder with nothing but pkg_name set.
    pub fn new(pkg_name: &str) -> TmplBuilder {
        TmplBuilder {
            pkg_name: pkg_name.to_owned(),
            pkg_type: None,
            pkg_info: None,
            deps: None,
        }
    }

    /// Initializes a new TmplBuilder from a PkgInfo. Useful for testing or as a shortcut
    pub fn from_pkg_info(pkg_info: PkgInfo) -> TmplBuilder {
        TmplBuilder {
            pkg_name: pkg_info.pkg_name.clone(),
            pkg_type: None,
            pkg_info: Some(pkg_info),
            deps: None,
        }
    }

    /// Gets the PkgType of the package of the TmplBuilder that's passed into the function
    pub fn get_type(&mut self) -> Result<&mut TmplBuilder, Error> {
        self.pkg_type = Some(figure_out_provider(&self.pkg_name)?);
        Ok(self)
    }

    /// Sets the PkgType of the package of the TmplBuilder that's passed into the function
    pub fn set_type(&mut self, pkg_type: PkgType) -> &mut TmplBuilder {
        self.pkg_type = Some(pkg_type);
        self
    }

    /// Gets the PkgInfo of the package of the TmplBuilder that's passed into the function
    pub fn get_info(&mut self) -> Result<&mut TmplBuilder, Error> {
        if self.pkg_type.is_some() {
            self.pkg_info = Some(get_pkginfo(&self.pkg_name, self.pkg_type.unwrap())?);
            Ok(self)
        } else {
            Err(Error::TooLittleInfo("Can't get PkgInfo without setting/getting PkgType first!".to_string()))
        }
    }

    /// Sets the PkgInfo of the package of the TmplBuilder that's passed into the function
    pub fn set_info(&mut self, pkg_info: PkgInfo) -> &mut TmplBuilder {
        self.pkg_info = Some(pkg_info);
        self
    }

    /// Gets the dependencies of the TmplBuilder that's passed into the function
    pub fn get_deps(&mut self) -> Result<&mut TmplBuilder, Error> {
        self.deps = if self.pkg_type == Some(PkgType::PerlDist) {
            Some(perldist_dep_graph(&self.pkg_name)?)
        } else if self.pkg_type == Some(PkgType::Gem) {
            Some(gem_dep_graph(&self.pkg_name)?)
        } else {
            None
        };

        Ok(self)
    }

    /// Checks if a Gem or PerlDist is built into Ruby/Perl.
    pub fn is_built_in(&self) -> Result<bool, Error> {
        if self.pkg_type.is_some() {
            let data: BuiltIns = serde_json::from_str(include_str!("built_in.in")).unwrap();

            let built_ins = BuiltIns {
                perl: data.perl,
                ruby: data.ruby,
            };

            if self.pkg_type.unwrap() == PkgType::Gem {
                for x in built_ins.ruby {
                    if self.pkg_name == x.name {
                        warn!(
                            "Gem {} is part of ruby, won't write a template for it!",
                            self.pkg_name
                        );
                        return Ok(true);
                    }
                }
            } else if self.pkg_type.unwrap() == PkgType::PerlDist {
                let pkg_name = self.pkg_name.replace("::", "-");

                for x in built_ins.perl {
                    if pkg_name == x.name {
                        warn!(
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
            Err(Error::TooLittleInfo("Can't check if Pkg is built in without setting/getting PkgType first!".to_string()))
        }
    }

    /// Updates a [Template](crate::types::Template)
    ///
    /// # Example
    /// ```
    /// use libtmplgen::{PkgInfo, Template, TmplBuilder};
    /// use std::fs::File;
    /// use std::io::prelude::*;
    ///
    /// // Do note that we only manually create a PkgInfo here to make the example easier to understand
    /// let pkg_info_crate = PkgInfo {
    ///        pkg_name: "tmplgen".to_string(),
    ///        version: "0.6.0".to_string(),
    ///        description: "Void Linux template generator for language-specific package managers"
    ///            .to_string(),
    ///        homepage: "https://github.com/Cogitri/tmplgen".to_string(),
    ///        license: vec!["GPL-3.0-or-later".to_string()],
    ///        dependencies: None,
    ///        sha: "afc403bf69ad4da168938961b0f02da86ef29d655967cfcbacc8201e1327aff4".to_string(),
    ///        download_url: Some(
    ///            "https://static.crates.io/crates/tmplgen/tmplgen-${version}.crate".to_string(),
    ///        ),
    /// };
    ///
    /// let mut old_template = Template { inner: String::new() };
    /// // Open whatever file you want to below.
    /// let mut file = File::open("src/lib/tests/template_test_crate.in").unwrap();
    /// file.read_to_string(&mut old_template.inner).unwrap();
    ///
    /// // This will return a [Template](crate::types::Template), which is updated to 0.6.0 (as specified by pkg_info_crate above)
    /// // Use TmplBuilder::new("tmplgen").get_info() to manually get the info we manually set in pkg_info_crate!
    /// // Won't update `homepage`, `distfiles` and `short_desc`, set the second argument to `true` for that
    /// let template_updated = TmplBuilder::from_pkg_info(pkg_info_crate).get_type().unwrap().update(&old_template, false).unwrap();
    ///
    /// // Write the [Template](crate::types::Template) to `./template`
    /// let mut file = File::create("./template").unwrap();
    /// file.write_all(template_updated.inner.as_bytes()).unwrap();
    /// ```
    pub fn update(&self, old_template: &Template, update_all: bool) -> Result<Template, Error> {
            let pkg_info = if self.pkg_info.is_some() {
                Ok(self.pkg_info.clone().unwrap())
            } else {
                Err(Error::TooLittleInfo("Can't update template without setting PkgInfo first!".to_string()))
            }?;

            info!("Updating template {}", &pkg_info.pkg_name);

                let mut template_string = old_template.inner.clone();

                let mut orig_ver_string = String::new();
                let mut orig_checksum_string = String::new();
                let mut orig_distfiles_string = String::new();

                for x in template_string.lines() {
                    if x.contains("version=") {
                        orig_ver_string = x.to_string();
                    }
                    if x.contains("checksum=") {
                        orig_checksum_string = x.to_string();
                    }
                    if x.contains("distfiles=") {
                        orig_distfiles_string = x.to_string();
                    }
                }

                template_string =
                    template_string.replace(&orig_ver_string, &format!("version={}", &pkg_info.version));

                if update_all {
                    let mut orig_homepage_string = String::new();
                    let mut orig_description_string = String::new();

                    for x in template_string.lines() {
                        if x.contains("homepage=") {
                            orig_homepage_string = x.to_string();
                        }
                        if x.contains("short_desc=") {
                            orig_description_string = x.to_string();
                        }
                    }

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
                    } else {
                        template_string = template_string.replace(
                            &orig_description_string,
                            &format!("short_desc=\"{}\"", &pkg_info.description),
                        );
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
                            &format!("checksum={}", gen_checksum(tmpl_download_url)?),
                        );
                    };
                }

            Ok(Template { inner: template_string.to_owned() })
    }

    /// Generates a new [Template](crate::types::Template)
    ///
    /// # Example
    ///
    /// ```
    /// use libtmplgen::{PkgInfo, PkgType, TmplBuilder};
    /// use std::fs::File;
    /// use std::io::prelude::*;
    ///
    /// // Do note that we only manually create a PkgInfo here to make the example easier to understand
    /// let pkg_info_crate = PkgInfo {
    ///        pkg_name: "tmplgen".to_string(),
    ///        version: "0.6.0".to_string(),
    ///        description: "Void Linux template generator for language-specific package managers"
    ///            .to_string(),
    ///        homepage: "https://github.com/Cogitri/tmplgen".to_string(),
    ///        license: vec!["GPL-3.0-or-later".to_string()],
    ///        dependencies: None,
    ///        sha: "afc403bf69ad4da168938961b0f02da86ef29d655967cfcbacc8201e1327aff4".to_string(),
    ///        download_url: Some(
    ///            "https://static.crates.io/crates/tmplgen/tmplgen-${version}.crate".to_string(),
    ///        ),
    /// };
    ///
    /// // Use TmplBuilder::new("tmplgen").get_type.generate() to do this automatically instead of
    /// // setting PkgInfo and PkgType manually
    /// let template = TmplBuilder::from_pkg_info(pkg_info_crate).set_type(PkgType::Crate).generate().unwrap();
    ///
    /// // Write the [Template](crate::types::Template) to `./template`
    /// let mut file = File::create("./template").unwrap();
    /// file.write_all(template.inner.as_bytes()).unwrap();
    /// ```
    ///
    /// # Errors
    /// * Errors out if any of the underlying functions fails
    pub fn generate(&self) -> Result<Template, Error> {


        let pkg_info = if self.pkg_info.is_some() {
            Ok(self.pkg_info.clone().unwrap())
        } else {
            Err(Error::TooLittleInfo("Can't write a new template without setting PkgInfo first!".to_string()))
        }?;

        // Can't have pkg_info without pkg_type, so we only have to check for pkg_info's existance
        let tmpl_type = self.pkg_type.unwrap();

        let template_in = include_str!("template.in");

        let maintainer = get_git_author()?;

        let mut license = String::new();

        for x in &pkg_info.license {
            if !license.is_empty() {
                license.push_str(", ");
            }

            license.push_str(&correct_license(x));
        }

        let mut description = check_string_len(&pkg_info.pkg_name, &pkg_info.description, "description");

        if description.chars().last().unwrap_or_default() == '.' {
            description.pop();
        }

        let mut template_string = template_in
            .replace("@version@", &pkg_info.version)
            .replace(
                "@description@",
                &description,
            )
            .replace("@license@", &license)
            .replace("@homepage@", &pkg_info.homepage)
            .replace("@maintainer@", &maintainer)
            .replace("@pkgname@", &pkg_info.pkg_name)
            .replace("@checksum@", &pkg_info.sha);

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
                template_string = template_string.replace("@makedepends@", &make_depends.trim_end());
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
            template_string = template_string.replace("\nhostmakedepends=\"@hostmakedepends@\"", "");
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
                .replace("@noarch@", "yes")
                .replace("@wrksrc@", "${pkgname/perl-/}-${version}");
        } else if tmpl_type == PkgType::Gem {
            template_string = template_string
                .replace("@build_style@", "gem")
                .replace("\nwrksrc=\"@wrksrc@\"", "")
                .replace("\nnoarch=@noarch@", "yes");
        } else {
            template_string = template_string
                .replace("@pkgname@", &pkg_info.pkg_name)
                .replace("\ndepends=\"@depends@\"", "")
                .replace("@build_style@", "cargo")
                .replace("\nwrksrc=\"@wrksrc@\"", "")
                .replace("\nnoarch=@noarch@", "");
        }

        let license = &pkg_info.license.join(", ");
        if license.contains(&"MIT".to_string())
            || license.contains(&"ISC".to_string())
            || license.contains(&"BSD".to_string())
            {
                template_string.push_str("\n\npost_install() {\n\tvlicense LICENSE\n}");
            }

        template_string.push_str("\n");

        Ok(Template { inner: template_string })
    }
}