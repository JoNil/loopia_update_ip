use env_logger::Env;
use public_ip_addr::get_public_ip;
use std::{collections::BTreeMap, env, time::Duration};
use tokio::task;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use xmlrpc::{Request, Value};

async fn update_ip(api_key: String) -> Result<(), Box<dyn std::error::Error>> {
    let ip = get_public_ip().await?;

    let update_result = task::spawn_blocking(move || {
        let request = Request::new("getZoneRecords")
            .arg("voysys@loopiaapi")
            .arg(api_key.clone())
            .arg("voysys.se")
            .arg("jonathan");

        let result = request.call_url("https://api.loopia.se/RPCSERV").unwrap();

        log::info!("Result: {:?}", result);

        let record = result
            .as_array()
            .ok_or("Did not receive array")
            .unwrap()
            .first()
            .ok_or("No element in array")
            .unwrap();

        let new_record = {
            let dns_type = record
                .get("type")
                .ok_or("Did not find type in response")
                .unwrap();
            let ttl = record
                .get("ttl")
                .ok_or("Did not find ttl in response")
                .unwrap();
            let priority = record
                .get("priority")
                .ok_or("Did not find priority in response")
                .unwrap();
            let record_id = record
                .get("record_id")
                .ok_or("Did not find record_id in response")
                .unwrap();

            let mut new_struct_map = BTreeMap::<String, Value>::new();
            new_struct_map.insert("type".to_owned(), dns_type.clone());
            new_struct_map.insert("ttl".to_owned(), ttl.clone());
            new_struct_map.insert("priority".to_owned(), priority.clone());
            new_struct_map.insert("rdata".to_owned(), Value::String(ip.to_string()));
            new_struct_map.insert("record_id".to_owned(), record_id.clone());

            Value::Struct(new_struct_map)
        };

        let update_request = Request::new("updateZoneRecord")
            .arg("voysys@loopiaapi")
            .arg(api_key)
            .arg("voysys.se")
            .arg("jonathan")
            .arg(new_record);

        update_request
            .call_url("https://api.loopia.se/RPCSERV")
            .unwrap()
    })
    .await;

    log::info!("Result: {:?}", update_result);

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), JobSchedulerError> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let api_key = env::var("API_KEY").unwrap();

    let sched = JobScheduler::new().await?;

    sched
        .add(Job::new_async("0 * * * * *", move |_uuid, _l| {
            Box::pin({
                let api_key = api_key.clone();
                async move {
                    if let Err(e) = update_ip(api_key).await {
                        log::error!("Error: {e:?}");
                    }
                }
            })
        })?)
        .await?;

    sched.start().await?;

    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
