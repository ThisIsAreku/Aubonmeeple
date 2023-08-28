use std::collections::HashMap;

use tokio_postgres::{Client, Error, NoTls, Row};

use crate::{
    frontend::{filter::Filters, server::State},
    game::{Deal, Game, Games, OkkazeoAnnounce, Reference, Review, Reviewer, Seller},
};

pub async fn connect_db() -> Result<Client, Error> {
    let db_url = "postgres://scrapy:scrapyscrapy@localhost/scraper";

    log::info!("[DB] connecting to DB");
    let (client, connection) = tokio_postgres::connect(db_url, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erreur de connexion: {}", e);
        }
    });

    Ok(client)
}

pub async fn delete_from_all_table_with_id(db_client: &Client, id: i32) -> Result<(), Error> {
    db_client
        .execute("DELETE FROM deal WHERE deal_oa_id = $1", &[&id])
        .await?;

    db_client
        .execute("DELETE FROM seller WHERE seller_oa_id = $1", &[&id])
        .await?;

    db_client
        .execute("DELETE FROM shipping WHERE ship_oa_id = $1", &[&id])
        .await?;

    db_client
        .execute("DELETE FROM reference WHERE ref_oa_id = $1", &[&id])
        .await?;

    db_client
        .execute("DELETE FROM reviewer WHERE reviewer_oa_id = $1", &[&id])
        .await?;

    db_client
        .execute("DELETE FROM okkazeo_announce WHERE oa_id = $1", &[&id])
        .await?;

    Ok(())
}

pub async fn insert_into_okkazeo_announce_table(
    db_client: &Client,
    game: &Box<Game>,
) -> Result<(), Error> {
    let okkazeo_insert_req = format!(
        r#"INSERT INTO okkazeo_announce ({}, {}, {}, {}, {}, {}, {}, {}, {}) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        "oa_id",
        "oa_last_modification_date",
        "oa_name",
        "oa_image",
        "oa_price",
        "oa_url",
        "oa_extension",
        "oa_barcode",
        "oa_city",
    );
    let _ = db_client
        .query(
            &okkazeo_insert_req,
            &[
                &(game.okkazeo_announce.id as i32),
                &game.okkazeo_announce.last_modification_date.unwrap(),
                &game.okkazeo_announce.name,
                &game.okkazeo_announce.image,
                &game.okkazeo_announce.price,
                &game.okkazeo_announce.url,
                &game.okkazeo_announce.extension,
                &(game.okkazeo_announce.barcode.unwrap_or_default() as i64),
                &game
                    .okkazeo_announce
                    .city
                    .as_ref()
                    .unwrap_or(&String::from("")),
            ],
        )
        .await?;

    Ok(())
}

pub async fn insert_into_shipping_table(
    db_client: &Client,
    id: i32,
    shipping: &HashMap<String, f32>,
) -> Result<(), Error> {
    let seller_insert_req = format!(
        r#"INSERT INTO shipping ({}, {}, {}) VALUES ($1, $2, $3)"#,
        "ship_oa_id", "ship_shipper", "ship_price",
    );

    for (key, value) in shipping.iter() {
        let _ = db_client
            .query(&seller_insert_req, &[&id, &key, &value])
            .await?;
    }

    Ok(())
}

pub async fn insert_into_seller_table(
    db_client: &Client,
    id: i32,
    seller: &Seller,
) -> Result<(), Error> {
    let seller_insert_req = format!(
        r#"INSERT INTO seller ({}, {}, {}, {}, {}) VALUES ($1, $2, $3, $4, $5)"#,
        "seller_oa_id", "seller_name", "seller_url", "seller_nb_announces", "seller_is_pro",
    );
    let _ = db_client
        .query(
            &seller_insert_req,
            &[
                &id,
                &seller.name,
                &seller.url,
                &(seller.nb_announces as i32),
                &seller.is_pro,
            ],
        )
        .await?;

    Ok(())
}
pub async fn insert_into_deal_table(db_client: &Client, id: i32, deal: &Deal) -> Result<(), Error> {
    let deal_insert_req = format!(
        r#"INSERT INTO deal ({}, {}, {}) VALUES ($1, $2, $3)"#,
        "deal_oa_id", "deal_price", "deal_percentage",
    );
    let _ = db_client
        .query(
            &deal_insert_req,
            &[&id, &deal.deal_price, &deal.deal_percentage],
        )
        .await?;

    Ok(())
}

pub async fn insert_into_reference_table(
    db_client: &Client,
    id: i32,
    references: &HashMap<String, Reference>,
) -> Result<(), Error> {
    let references_insert_req = format!(
        r#"INSERT INTO reference ({}, {}, {}, {}) VALUES ($1, $2, $3, $4)"#,
        "ref_oa_id", "ref_name", "ref_price", "ref_url",
    );

    for val in references.values() {
        let _ = db_client
            .query(
                &references_insert_req,
                &[&id, &val.name, &val.price, &val.url],
            )
            .await?;
    }
    Ok(())
}

pub async fn insert_into_reviewer_table(
    db_client: &Client,
    id: i32,
    reviewers: &HashMap<std::string::String, Reviewer>,
) -> Result<(), Error> {
    let references_insert_req = format!(
        r#"INSERT INTO reviewer ({}, {}, {}, {}, {}) VALUES ($1, $2, $3, $4, $5)"#,
        "reviewer_oa_id", "reviewer_name", "reviewer_url", "reviewer_note", "reviewer_number",
    );

    for val in reviewers.values() {
        let _ = db_client
            .query(
                &references_insert_req,
                &[&id, &val.name, &val.url, &val.note, &(val.number as i32)],
            )
            .await?;
    }
    Ok(())
}

pub async fn insert_announce_into_db(db_client: &Client, game: &Box<Game>) -> Result<(), Error> {
    log::debug!("inserting {} into DB ", game.okkazeo_announce.name);
    let id = game.okkazeo_announce.id as i32;
    insert_into_okkazeo_announce_table(db_client, game).await?;
    insert_into_seller_table(db_client, id, &game.okkazeo_announce.seller).await?;
    insert_into_shipping_table(db_client, id, &game.okkazeo_announce.shipping).await?;
    insert_into_deal_table(db_client, id, &game.deal).await?;
    insert_into_reference_table(db_client, id, &game.references).await?;
    insert_into_reviewer_table(db_client, id, &game.review.reviews).await?;

    Ok(())
}

pub async fn update_game_from_db(db_client: &Client, game: &Game) -> Result<(), Error> {
    let references_insert_req = format!(
        r#"UPDATE okkazeo_announce SET {} = $1, {} = $2 WHERE {} = $3"#,
        "oa_last_modification_date", "oa_price", "oa_id",
    );

    let _ = db_client
        .query(
            &references_insert_req,
            &[
                &game.okkazeo_announce.last_modification_date,
                &game.okkazeo_announce.price,
                &(game.okkazeo_announce.id as i32),
            ],
        )
        .await?;
    Ok(())
}

pub async fn craft_game_from_row(db_client: &Client, row: Row) -> Result<Game, Error> {
    let id: i32 = row.try_get("oa_id")?;
    let nb_announces: i32 = row.try_get("seller_nb_announces")?;

    let game = Game {
        okkazeo_announce: OkkazeoAnnounce {
            id: id as u32,
            name: row.try_get("oa_name")?,
            image: row.try_get("oa_image")?,
            price: row.try_get("oa_price")?,
            url: row.try_get("oa_url")?,
            extension: row.try_get("oa_extension").unwrap_or_default(),
            shipping: select_shipping_from_db(db_client, id).await?,
            seller: Seller {
                name: row.try_get("seller_name")?,
                url: row.try_get("seller_url")?,
                nb_announces: nb_announces as u32,
                is_pro: row.try_get("seller_is_pro")?,
            },
            barcode: match row.try_get::<&str, i64>("oa_barcode") {
                Ok(v) => Some(v as u64),
                Err(_) => None,
            },
            city: row.try_get("oa_city")?,
            last_modification_date: row.try_get("oa_last_modification_date")?,
        },
        references: select_references_from_db(db_client, id).await?,
        review: select_reviews_from_db(db_client, id).await?,
        deal: Deal {
            deal_price: row.try_get("deal_price")?,
            deal_percentage: row.try_get("deal_percentage")?,
        },
    };

    Ok(game)
}

pub async fn select_game_with_id_from_db(db_client: &Client, id: u32) -> Option<Game> {
    log::debug!("[DB] select game with id from db : {}", id);
    let select_req = format!(
        "SELECT *
                FROM okkazeo_announce oa
                JOIN deal d on d.deal_oa_id = oa.oa_id
                JOIN seller s on s.seller_oa_id = oa.oa_id
                WHERE oa.oa_id = $1"
    );

    let res = db_client.query(&select_req, &[&(id as i32)]).await.unwrap();

    let row = res.into_iter().next()?;

    match craft_game_from_row(db_client, row).await {
        Ok(game) => {
            log::debug!("[DB] game crafted from DB: {:#?}", game);
            Some(game)
        }
        Err(e) => {
            log::error!("[DB] craft game from row error for id {} : {}", id, e);
            None
        }
    }
}

/*FROM okkazeo_announce oa
JOIN deal d on d.deal_oa_id = oa.oa_id
JOIN seller s on s.seller_oa_id = oa.oa_id
JOIN reviewer r on r.reviewer_oa_id = oa.oa_id
GROUP BY oa.oa_id, oa.oa_last_modification_date, oa.oa_name, oa.oa_image, oa.oa_price, oa.oa_url, oa.oa_extension, oa.oa_barcode, oa.oa_city
ORDER BY avg_review_note DESC;*/

pub async fn select_games_from_db(db_client: &Client, state: &State) -> Result<Games, Error> {
    let order_by = match state.sort.sort.as_str() {
        "deal" => "d.deal_price ASC",
        _ => "oa.oa_last_modification_date DESC",
    };

    let select_req = format!(
        "SELECT * 
                FROM okkazeo_announce oa 
                JOIN deal d on d.deal_oa_id = oa.oa_id
                JOIN seller s on s.seller_oa_id = oa.oa_id
                WHERE oa.oa_name ilike $1 AND oa.oa_city ilike $2
                ORDER BY {} LIMIT $3 OFFSET $4",
        order_by
    );

    let res = db_client
        .query(
            &select_req,
            &[
                &format!(
                    "%{}%",
                    state.filters.name.as_ref().unwrap_or(&String::new())
                ),
                &format!(
                    "%{}%",
                    state.filters.city.as_ref().unwrap_or(&String::new())
                ),
                &(state.pagination.per_page as i64),
                &((state.pagination.page * state.pagination.per_page) as i64),
            ],
        )
        .await?;

    let mut games = Games {
        ..Default::default()
    };
    for row in res {
        let game = match craft_game_from_row(db_client, row).await {
            Ok(game) => {
                log::debug!("[DB] game crafted from DB: {:#?}", game);
                game
            }
            Err(e) => {
                log::error!("[DB] craft game from row error : {}", e);
                return Err(e);
            }
        };
        games.games.push(Box::new(game))
    }

    Ok(games)
}

pub async fn select_count_filtered_games_from_db(
    db_client: &Client,
    filters: Filters,
) -> Result<i64, Error> {
    let select_req = format!(
        "SELECT COUNT(*)
                FROM okkazeo_announce oa
                JOIN deal d on d.deal_oa_id = oa.oa_id
                WHERE oa.oa_name ilike $1 AND oa.oa_city ilike $2"
    );

    let res = db_client
        .query(
            &select_req,
            &[
                &format!("%{}%", filters.name.unwrap_or_default()),
                &format!("%{}%", filters.city.unwrap_or_default()),
            ],
        )
        .await?;

    let nbr: i64 = res.get(0).unwrap().try_get(0)?;

    Ok(nbr)
}

pub async fn select_shipping_from_db(
    db_client: &Client,
    id: i32,
) -> Result<HashMap<String, f32>, Error> {
    let select_req = format!(
        "SELECT *
                FROM shipping
                WHERE ship_oa_id = $1"
    );

    let res = db_client.query(&select_req, &[&id]).await?;

    let mut ships = HashMap::<String, f32>::new();
    for row in res {
        let shipper = row.try_get("ship_shipper")?;
        let price = row.try_get("ship_price")?;
        ships.insert(shipper, price);
    }

    Ok(ships)
}

pub async fn select_all_ids_from_oa_table_from_db(db_client: &Client) -> Result<Vec<i32>, Error> {
    let select_req = format!(
        "SELECT oa_id
                FROM okkazeo_announce"
    );

    let res = db_client.query(&select_req, &[]).await?;

    res.into_iter().map(|row| row.try_get("oa_id")).collect()
}

pub async fn select_references_from_db(
    db_client: &Client,
    id: i32,
) -> Result<HashMap<String, Reference>, Error> {
    let select_req = format!(
        "SELECT *
                FROM reference
                WHERE ref_oa_id = $1"
    );

    let res = db_client.query(&select_req, &[&id]).await?;

    let mut refs = HashMap::<String, Reference>::new();
    for row in res {
        let name: String = row.try_get("ref_name")?;
        let price = row.try_get("ref_price")?;
        let url = row.try_get("ref_url")?;
        refs.insert(name.clone(), Reference { name, price, url });
    }

    Ok(refs)
}

pub async fn select_reviews_from_db(db_client: &Client, id: i32) -> Result<Review, Error> {
    let select_req = format!(
        "SELECT *
                FROM reviewer
                WHERE reviewer_oa_id = $1"
    );

    let res = db_client.query(&select_req, &[&id]).await?;

    let mut revs = HashMap::<String, Reviewer>::new();
    for row in res {
        let name: String = row.try_get("reviewer_name")?;
        let url = row.try_get("reviewer_url")?;
        let note = row.try_get("reviewer_note")?;
        let number: i32 = row.try_get("reviewer_number")?;
        revs.insert(
            name.clone(),
            Reviewer {
                name,
                url,
                note,
                number: number as u32,
            },
        );
    }

    let mut rev = Review {
        reviews: revs,
        average_note: 0.0,
    };
    rev.compute_average_note();

    Ok(rev)
}
