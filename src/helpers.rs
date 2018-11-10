use types::*;

pub fn missing_field_s(field_name: &str) -> String {
    error!(
        "Couldn't determine field '{}'! Please add it to the template yourself.",
        field_name
    );

    String::from("")
}

pub fn missing_field_v(field_name: &str) -> Vec<String> {
    error!(
        "Couldn't determine field '{}'! Please add it to the template yourself.",
        field_name
    );

    vec![String::from("")]
}

pub fn figure_out_provider(
    tmpl_type: Option<PkgType>,
    pkg_name: &String,
) -> Result<PkgType, String> {
    if tmpl_type.is_none() {
        if crates_io_api::SyncClient::new()
            .get_crate(&pkg_name)
            .is_ok()
        {
            debug!("Determined the target package to be a crate");
            Ok(PkgType::Crate)
        } else if rubygems_api::SyncClient::new()
            .gem_info(&pkg_name)
            .is_ok()
        {
            debug!("Determined the target package to be a ruby gem");
            Ok(PkgType::Gem)
        } else {
            Err("Unable to determine what type of the target package, please specify it via the '-t' parameter!".to_string())
        }
    } else {
        Ok(tmpl_type.unwrap())
    }
}
