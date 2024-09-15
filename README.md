### Wprowadzenie

Tworzenie portalu WWW w technologii Web3 z wykorzystaniem języka **Rust** oraz **smart kontraktów** do autoryzacji użytkowników to zaawansowane zadanie, które łączy programowanie backendowe z technologią blockchain. Poniżej przedstawiam krok po kroku, jak zrealizować taki projekt.

### 1. Wybór platformy blockchain

Ponieważ Rust jest używany w wielu projektach blockchain, masz kilka opcji:

- **Ethereum**: Główna platforma dla smart kontraktów. Smart kontrakty są pisane w **Solidity**, ale możesz użyć Rust do interakcji z nimi.
- **Solana**: Wspiera pisanie smart kontraktów bezpośrednio w Rust.
- **Polkadot/Substrate**: Framework do tworzenia własnych blockchainów w Rust.

W tym przewodniku skupimy się na **Solanie**, ponieważ umożliwia pisanie smart kontraktów w Rust, co ułatwia integrację.

### 2. Przygotowanie środowiska

#### Instalacja Rust

Upewnij się, że masz zainstalowany Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Instalacja Solana CLI

Zainstaluj narzędzia wiersza poleceń Solany:

```bash
sh -c "$(curl -sSfL https://release.solana.com/v1.10.32/install)"
```

#### Instalacja Anchor

**Anchor** to framework ułatwiający tworzenie smart kontraktów na Solanie.

```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest
```

### 3. Tworzenie smart kontraktu

#### Inicjalizacja projektu Anchor

Utwórz nowy projekt:

```bash
anchor init my_auth_app
cd my_auth_app
```

#### Implementacja smart kontraktu

Edytuj plik `programs/my_auth_app/src/lib.rs`:

```rust
use anchor_lang::prelude::*;

declare_id!("YourProgramIDHere");

#[program]
pub mod my_auth_app {
    use super::*;

    pub fn register_user(ctx: Context<RegisterUser>, username: String) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        user_account.authority = ctx.accounts.authority.key();
        user_account.username = username;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct RegisterUser<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 40)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct UserAccount {
    pub authority: Pubkey,
    pub username: String,
}
```

### 4. Kompilacja i wdrożenie smart kontraktu

Kompiluj kontrakt:

```bash
anchor build
```

Uruchom lokalny klaster Solany:

```bash
solana-test-validator
```

Wdroż kontrakt:

```bash
anchor deploy
```

Zapamiętaj adres programu wyświetlony po wdrożeniu.

### 5. Tworzenie backendu w Rust

#### Dodanie zależności

Edytuj `Cargo.toml` w katalogu głównym:

```toml
[dependencies]
actix-web = "4"
solana-client = "1.10.32"
solana-sdk = "1.10.32"
tokio = { version = "1", features = ["full"] }
```

#### Implementacja serwera

Utwórz plik `src/main.rs`:

```rust
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
```

### 6. Tworzenie frontendu

Możesz użyć frameworka **Yew** do tworzenia frontendu w Rust.

#### Inicjalizacja projektu Yew

```bash
cargo new my_auth_app_frontend
cd my_auth_app_frontend
```

Edytuj `Cargo.toml`:

```toml
[dependencies]
yew = { version = "0.20", features = ["csr"] }
wasm-bindgen = "0.2"
```

#### Implementacja frontendu

Edytuj `src/main.rs`:

```rust
use yew::prelude::*;
use reqwest::Client;

#[function_component(App)]
fn app() -> Html {
    let username = use_state(|| "".to_string());
    let onsubmit = {
        let username = username.clone();
        Callback::from(move |e: FocusEvent| {
            e.prevent_default();
            let username = username.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let client = Client::new();
                let res = client.post("http://127.0.0.1:8080/register")
                    .json(&serde_json::json!({ "username": *username }))
                    .send()
                    .await;
                // Obsługa odpowiedzi
            });
        })
    };

    html! {
        <form {onsubmit}>
            <input
                type="text"
                value={(*username).clone()}
                oninput={Callback::from(move |e: InputEvent| {
                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                    username.set(input.value());
                })}
            />
            <button type="submit">{ "Register" }</button>
        </form>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
```

#### Budowa frontendu

Użyj narzędzia `trunk` do budowy aplikacji:

```bash
cargo install trunk
trunk serve
```

### 7. Integracja i testowanie

- Upewnij się, że backend działa na `127.0.0.1:8080`.
- Uruchom frontend i przetestuj rejestrację użytkownika.
- Sprawdź węzeł Solany, aby zobaczyć transakcje.

### 8. Bezpieczeństwo

- **Klucze prywatne**: Nigdy nie przechowuj kluczy prywatnych w kodzie źródłowym. Użyj bezpiecznych metod przechowywania, takich jak zmienne środowiskowe lub usługi zarządzania kluczami.
- **Walidacja danych**: Upewnij się, że wszystkie dane wejściowe od użytkowników są odpowiednio walidowane.
- **HTTPS**: Wdrażaj backend z użyciem HTTPS dla bezpieczeństwa transmisji danych.

### 9. Wdrożenie na sieć testową lub mainnet

Gdy aplikacja działa lokalnie, możesz wdrożyć smart kontrakt na sieć testową Solany (devnet):

```bash
solana config set --url https://api.devnet.solana.com
anchor deploy
```

Dostosuj aplikację, aby używała odpowiedniego URL RPC.

### Podsumowanie

Stworzenie portalu WWW w Rust z wykorzystaniem Web3 i smart kontraktów do autoryzacji użytkowników wymaga integracji wielu technologii. Korzystając z Solany i Anchor, możesz pisać smart kontrakty w Rust, co ułatwia cały proces. Pamiętaj o najlepszych praktykach bezpieczeństwa i dokładnym testowaniu aplikacji przed wdrożeniem na produkcję.
