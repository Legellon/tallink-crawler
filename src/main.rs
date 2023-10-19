use anyhow::{anyhow, Result};

const API_ID: &'static str = "28304981";
const API_HASH: &'static str = "f6aa92e440d7178475f8d7b6c9b51232";
const BOT_TOKEN: &'static str = "6499347160:AAGlUaC2m5lwv8Je2kUVLeIQcmkRHhoJJQM";

const IGOR_CHAT_ID: &'static str = "823545251";
const MY_CHAT_ID: &'static str = "439451757";

const TRIPS_ENDPOINT: &'static str = "https://booking.tallink.com/api/cruise/timetables?from=tal&to=sto&dateFrom=2023-09-11&dateTo=2023-09-24&voyageType=CRUISE";

struct TicketStatus {
    in_stock: bool,
}

async fn send_chat_telegram_message(msg: &str, chat_id: &str) -> Result<()> {
    let uri = format!(
        "https://api.telegram.org/bot{}/sendMessage?chat_id={}&parse_mode=Markdown&text={}",
        BOT_TOKEN, chat_id, msg
    );
    let _ = reqwest::get(uri).await?;
    Ok(())
}

async fn get_ticket_status(ticket: &str, max_price: i64) -> Result<TicketStatus> {
    let body = reqwest::get(TRIPS_ENDPOINT)
        .await?
        .json::<serde_json::Value>()
        .await?;

    let ticket_body = body
        .get("trips")
        .ok_or(anyhow!("missing trips info in body"))?
        .get(ticket)
        .ok_or(anyhow!("missing the target in trips content"))?;

    let is_disabled = ticket_body
        .get("disabled")
        .ok_or(anyhow!("missing `disabled` field in target content"))?
        .as_bool()
        .unwrap_or(true);

    let is_price_satisfied = ticket_body
        .get("personPrice")
        .ok_or(anyhow!("missing `personPrice` field` in target content"))?
        .as_i64()
        .map_or(false, |v| v <= max_price);

    Ok(TicketStatus {
        in_stock: !is_disabled && is_price_satisfied,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    use tokio::time;

    let ticket = "2023-09-15";
    let max_price = 200;
    let interval_time = 30;

    let mut interval = time::interval(time::Duration::from_secs(interval_time));
    let mut i = 0;

    loop {
        i += 1;
        interval.tick().await;
        match get_ticket_status(ticket, max_price).await {
            Ok(TicketStatus { in_stock: true }) => {
                println!("info: found ticket");
                let msg = format!("Ticket In Stock - {}", ticket);
                send_chat_telegram_message(&msg, MY_CHAT_ID).await?;
            }
            Err(e) => eprintln!("err: {}", e),
            _ => {}
        }
        println!("info: {} requests made ({} secs interval)", i, interval_time);
    }
}
