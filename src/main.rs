use regex::Regex;
use steamworks::{Client, SteamError, QueryResult, DownloadItemResult};
use steamworks::{PublishedFileId, QueryResults};

fn on_item_downloaded(result: DownloadItemResult, client: &Client, item: &QueryResult) {

    // TODO: extraction of files

    std::process::exit(0);
}

fn download_item(client: &Client, item: &QueryResult) {
    let ugc = client.ugc();
    match ugc.item_install_info(item.published_file_id) {
        Some(info) => {
            println!("[steam-workshop-downloader] found existing install of {} at {}", item.title, info.folder);
            on_item_downloaded(DownloadItemResult {
                app_id: item.consumer_app_id.unwrap(),
                error: None,
                published_file_id: item.published_file_id,
            }, &client, &item);
        },
        None => {
            let downloaded = ugc.download_item(item.published_file_id, true);
            if !downloaded {
                panic!("[steam-workshop-downloader] failed to download item {}", item.published_file_id.0);
            }

            // TODO: fix reference lifetimes here
            client.register_callback(|result: DownloadItemResult| on_item_downloaded(result, &client, &item));

            let download_info = ugc.item_download_info(item.published_file_id);
            if let Some((mut bytes_downloaded, mut total_bytes)) = download_info {
                while bytes_downloaded < total_bytes {
                    ::std::thread::sleep(::std::time::Duration::from_millis(100));

                    println!("[steam-workshop-downloader] downloading item {} ({}%)", item.published_file_id.0, bytes_downloaded / total_bytes);

                    let info = ugc.item_download_info(item.published_file_id);
                    if !info.is_some() {
                        break;
                    }

                    (bytes_downloaded, total_bytes) = info.unwrap();
                }
            }
        }
    }
}

fn on_item_queried(client: &Client, res: &Result<QueryResults, SteamError>) {
    match res {
        Ok(results) => {
            let res = results.iter()
                .filter_map(std::convert::identity)
                .find(|i| i.title.len() > 0)
                .unwrap();

            download_item(&client, &res);
        },
        Err(err) => panic!("[steam-workshop-downloader] {:?}", err),
    }
}

fn get_input_workshop_id() -> u64 {
    let args = std::env::args().skip(1);
    let mut url = String::new();
    for arg in args {
        url.push_str(&arg);
    }

    let regex_url = Regex::new(r"https://steamcommunity\.com/sharedfiles/filedetails/\?id=").unwrap();
    url = regex_url.replace(&url, "").to_string();
    url.parse::<u64>().expect("[steam-workshop-downloader] invalid url or workshop id")
}

fn main() {
    let item_id = PublishedFileId::from(get_input_workshop_id());
    match Client::init_app(4000) {
        Ok(steam) => {
            let (client, single) = steam;
            let ugc = client.ugc();
            ugc.query_item(item_id).unwrap()
                .include_metadata(true)
                .fetch(| res | on_item_queried(&client, &res));

            println!("[steam-workshop-downloader] waiting for item to be queried...");
            loop {
                single.run_callbacks();
                ::std::thread::sleep(::std::time::Duration::from_millis(100));
            }
        },
        Err(err) => panic!("[steam-workshop-downloader] {:?}", err),
    }
}
