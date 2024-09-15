use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    signer::Signer,
    transaction::Transaction,
    pubkey::Pubkey,
    signature::Keypair,
    system_program,
};
use std::str::FromStr;

async fn register_user(data: web::Json<RegisterInfo>) -> impl Responder {
    let rpc_url = "http://localhost:8899";
    let client = RpcClient::new(rpc_url);

    // Załaduj klucz prywatny (uwaga na bezpieczeństwo)
    let payer = Keypair::new(); // W produkcji użyj bezpiecznego przechowywania kluczy

    let program_id = Pubkey::from_str("YourProgramIDHere").unwrap();

    // Tworzenie instrukcji
    let user_account = Keypair::new();
    let username = data.username.clone();

    let accounts = vec![
        // Konta wymagane przez smart kontrakt
    ];

    let instruction = Instruction {
        program_id,
        accounts,
        data: vec![], // Serializowane dane wejściowe
    };

    // Tworzenie transakcji
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer, &user_account],
        client.get_latest_blockhash().unwrap(),
    );

    // Wysłanie transakcji
    match client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => HttpResponse::Ok().body(format!("User registered: {}", signature)),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error: {}", err)),
    }
}

#[derive(serde::Deserialize)]
struct RegisterInfo {
    username: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/register", web::post().to(register_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
