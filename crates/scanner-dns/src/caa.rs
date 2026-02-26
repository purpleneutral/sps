use hickory_resolver::TokioResolver;
use scanner_core::check::CheckResult;
use scanner_core::spec::Category;

const CAT: Category = Category::EmailDnsSecurity;

/// Check if a CAA record is present restricting certificate issuance.
pub async fn check_caa(domain: &str, resolver: &TokioResolver) -> CheckResult {
    match resolver
        .lookup(domain, hickory_proto::rr::RecordType::CAA)
        .await
    {
        Ok(response) => {
            let records: Vec<String> = response.iter().map(|r| r.to_string()).collect();
            if records.is_empty() {
                CheckResult::fail(
                    CAT,
                    "caa_present",
                    "CAA record present",
                    1,
                    Some("No CAA records found".into()),
                )
            } else {
                CheckResult::pass(
                    CAT,
                    "caa_present",
                    "CAA record present",
                    1,
                    Some(format!("{} CAA record(s) found", records.len())),
                )
            }
        }
        Err(_) => CheckResult::fail(
            CAT,
            "caa_present",
            "CAA record present",
            1,
            Some("No CAA records found".into()),
        ),
    }
}
