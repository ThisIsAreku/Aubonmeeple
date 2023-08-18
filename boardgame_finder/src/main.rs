use game::{Game, Games, Reference};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time;
use website::agorajeux::get_agorajeux_price_and_url_by_name;
use website::bgg::get_bgg_note;
use website::knapix::get_knapix_prices;
use website::ludocortex::get_ludocortex_price_and_url;
use website::okkazeo::{get_atom_feed, get_okkazeo_barcode_and_city};
use website::philibert::get_philibert_price_and_url;
use website::trictrac::get_trictrac_note;
use website::ultrajeux::get_ultrajeux_price_and_url;

mod game;
mod server;
mod website;

async fn parse_game_feed(games: &mut Arc<std::sync::Mutex<Games>>) {
    let feed = get_atom_feed().await.unwrap();
    'outer: for entry in feed.entries {
        //println!("entry: {:#?}", entry);

        let id = entry.id.parse::<u32>().unwrap();
        for g in games.lock().unwrap().games.iter() {
            if g.id == id {
                continue 'outer;
            }
        }

        let title = entry.title.unwrap();

        let mut vec_name = title.content.split('-').collect::<Vec<&str>>();
        let _ = vec_name.pop();
        let name = vec_name.join("-").trim().to_string();
        let mut result = String::new();
        let mut inside_parentheses = false;

        for c in name.chars() {
            match c {
                '(' => inside_parentheses = true,
                ')' => inside_parentheses = false,
                _ if !inside_parentheses => result.push(c),
                _ => (),
            }
        }

        let mut game = Game {
            id,
            name,
            references: HashMap::<String, Reference>::new(),
            last_modification_date: entry.updated,
            ..Default::default()
        };

        let okkazeo_reference = Reference {
            name: String::from("okkazeo"),
            url: entry.links.first().cloned().unwrap().href,
            price: entry
                .summary
                .unwrap()
                .content
                .split('>')
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .split('€')
                .collect::<Vec<&str>>()
                .first()
                .unwrap()
                .parse::<f32>()
                .unwrap(),
        };
        game.references
            .insert(String::from("okkazeo"), okkazeo_reference);

        (game.barcode, game.city) = get_okkazeo_barcode_and_city(game.id).await;

        get_knapix_prices(&mut game).await;

        if game.references.get("philibert").is_none() {
            if let Some((price, url)) = get_philibert_price_and_url(&game.name, game.barcode).await
            {
                game.references.insert(
                    "philibert".to_string(),
                    Reference {
                        name: "philibert".to_string(),
                        price,
                        url,
                    },
                );
            }
        }
        if game.references.get("agorajeux").is_none() {
            if let Some((price, url)) = get_agorajeux_price_and_url_by_name(&game.name).await {
                game.references.insert(
                    "agorajeux".to_string(),
                    Reference {
                        name: "agorajeux".to_string(),
                        price,
                        url,
                    },
                );
            }
        }

        if game.references.get("ultrajeux").is_none() {
            if let Some((price, url)) = get_ultrajeux_price_and_url(&game.name, game.barcode).await
            {
                game.references.insert(
                    "ultrajeux".to_string(),
                    Reference {
                        name: "ultrajeux".to_string(),
                        price,
                        url,
                    },
                );
            }
        }

        if game.references.get("ludocortex").is_none() {
            if let Some((price, url)) = get_ludocortex_price_and_url(&game.name, game.barcode).await
            {
                game.references.insert(
                    "ludocortex".to_string(),
                    Reference {
                        name: "ludocortex".to_string(),
                        price,
                        url,
                    },
                );
            }
        }

        let note = get_trictrac_note(&game.name).await;
        if note.is_some() {
            (game.note_trictrac, game.review_count_trictrac) = note.unwrap();
        } else {
            println!("Cannot get trictrac note");
        }

        let note = get_bgg_note(&game.name).await;
        if note.is_some() {
            (game.note_bgg, game.review_count_bgg) = note.unwrap();
        } else {
            println!("Cannot get bgg note");
        }

        //println!("{:#?}", game);
        let mut locked_games = games.lock().unwrap();
        match locked_games.games.binary_search(&game) {
            Ok(_) => {} // element already in vector @ `pos`
            Err(pos) => locked_games.games.insert(pos, game),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    let games = Arc::new(Mutex::new(Games::new()));
    let interval = Duration::from_secs(60 * 5); // Remplacez X par le nombre de minutes souhaité

    let mut game_clone = games.clone();
    let _ = tokio::spawn(async move { server::set_server(&mut game_clone).await });

    loop {
        let start = Instant::now();
        parse_game_feed(&mut games.clone()).await;
        let duration = start.elapsed();

        if duration < interval {
            time::sleep(interval - duration).await;
        }
    }
}
