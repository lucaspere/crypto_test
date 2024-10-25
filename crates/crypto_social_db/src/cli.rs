use inquire::{Select, Text};
use rust_decimal::prelude::FromPrimitive;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn run_cli(db: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let choices = vec![
            "Create User",
            "Follow User",
            "Unfollow User",
            "List Followers",
            "List Following",
            "Create Token Call",
            "List Token Calls",
            "Exit",
        ];

        let choice = Select::new("Select an action:", choices).prompt()?;

        match choice {
            "Create User" => create_user(db).await?,
            "Follow User" => follow_user(db).await?,
            "Unfollow User" => unfollow_user(db).await?,
            "List Followers" => list_followers(db).await?,
            "List Following" => list_following(db).await?,
            "Create Token Call" => create_token_call(db).await?,
            "List Token Calls" => list_token_calls(db).await?,
            "Exit" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn print_menu() {
    println!("\n--- CLI Menu ---");
    println!("1. Create User");
    println!("2. Follow User");
    println!("3. Unfollow User");
    println!("4. List Followers");
    println!("5. List Following");
    println!("6. Create Token Call");
    println!("7. List Token Calls");
    println!("8. Exit");
}

fn get_user_input(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let input = Text::new(prompt).prompt()?;
    Ok(input)
}

async fn create_user(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let username = Text::new("Enter username:").prompt()?;
    let telegram_id = Text::new("Enter telegram ID (optional):").prompt()?;

    let user = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(username),
        telegram_id: Set(if telegram_id.is_empty() {
            None
        } else {
            Some(telegram_id)
        }),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };

    let inserted_user = Users::insert(user).exec(db).await?;
    println!("User created with ID: {}", inserted_user.last_insert_id);
    Ok(())
}

async fn follow_user(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let follower_id = Uuid::parse_str(&get_user_input("Enter follower ID: ")?)?;
    let followed_id = Uuid::parse_str(&get_user_input("Enter followed ID: ")?)?;

    user_follow::follow_user(db, follower_id, followed_id).await?;
    println!("User followed successfully");
    Ok(())
}

async fn unfollow_user(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let follower_id = Uuid::parse_str(&get_user_input("Enter follower ID: ")?)?;
    let followed_id = Uuid::parse_str(&get_user_input("Enter followed ID: ")?)?;

    user_follow::unfollow_user(db, follower_id, followed_id).await?;
    println!("User unfollowed successfully");
    Ok(())
}

async fn list_followers(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let user_id = Uuid::parse_str(&get_user_input("Enter user ID: ")?)?;

    let followers = user_follow::get_followers(db, user_id).await?;
    println!("Followers:");
    for follower in followers {
        println!("- {} ({})", follower.username, follower.id);
    }
    Ok(())
}

async fn list_following(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let user_id = Uuid::parse_str(&get_user_input("Enter user ID: ")?)?;

    let following = user_follow::get_following(db, user_id).await?;
    println!("Following:");
    for followed in following {
        println!("- {} ({})", followed.username, followed.id);
    }
    Ok(())
}

async fn create_token_call(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let token_address = get_user_input("Enter token address: ")?;
    let user_id = Uuid::parse_str(&get_user_input("Enter user ID: ")?)?;
    let call_type = get_user_input("Enter call type (buy/sell): ")?;
    let price_at_call = get_user_input("Enter price at call: ")?.parse::<f64>()?;
    let target_price = get_user_input("Enter target price (optional): ")?;

    let token_call = token_calls::ActiveModel {
        token_address: Set(token_address),
        user_id: Set(user_id),
        call_type: Set(call_type),
        price_at_call: Set(rust_decimal::Decimal::from_f64(price_at_call).unwrap()),
        target_price: Set(if target_price.is_empty() {
            None
        } else {
            Some(rust_decimal::Decimal::from_f64(target_price.parse::<f64>()?).unwrap())
        }),
        call_date: Set(chrono::Utc::now().into()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };

    let inserted_call = TokenCalls::insert(token_call).exec(db).await?;
    println!(
        "Token call created with ID: {}",
        inserted_call.last_insert_id
    );
    Ok(())
}

async fn list_token_calls(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let token_calls = TokenCalls::find().all(db).await?;
    println!("Token Calls:");
    for call in token_calls {
        println!(
            "- ID: {}, Token: {}, User: {}, Type: {}, Price: {}",
            call.id, call.token_address, call.user_id, call.call_type, call.price_at_call
        );
    }
    Ok(())
}
