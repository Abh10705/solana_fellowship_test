use actix_web::{web, App, HttpResponse, HttpServer, Responder, ResponseError};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    instruction::{Instruction, AccountMeta},
    pubkey::Pubkey,
    signature::{Keypair, Signer, Signature},
    system_instruction,
    signer::SignerError,
    program_error::ProgramError,
};
use spl_token::instruction as spl_instruction;
use spl_associated_token_account::get_associated_token_address;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use bs58;
use std::str::FromStr;
use log::{info, error};
use env_logger;
use anyhow::Error as AnyhowError;
// ALL THE CRATES WHICH WE NEED IG
#[derive(Debug, Serialize, Deserialize)]
struct KeypairGenResponse {
    pubkey: String,
    secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenCreateRequest {
    #[serde(rename = "mintAuthority")]
    mint_authority: String,
    mint: String,
    decimals: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenMintRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageSignRequest {
    message: String,
    secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageVerifyRequest {
    message: String,
    signature: String,
    pubkey: String,
}

#[derive(Debug, Serialize, Deserialize)]git remote set-url origin https://github.com/Abh10705/solana_fellowship_test.git
struct MessageSignResponse {
    signature: String,
    public_key: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageVerifyResponse {
    valid: bool,
    message: String,
    pubkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InstructionAccountDetails {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct InstructionOutput {
    program_id: String,
    accounts: Vec<InstructionAccountDetails>,
    instruction_data: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceResponse<T> {
    success: bool,
    data: T,
}

#[derive(Debug, Serialize)]
struct ServiceErrorResponse {
    success: bool,
    error: String,
}
//ranodm structs ez(hopefully this isnt wrong)
#[derive(Debug)]
struct SolanaApiError(String);

impl std::fmt::Display for SolanaApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<AnyhowError> for SolanaApiError {
    fn from(_: AnyhowError) -> Self {
        SolanaApiError("Missing required fields".to_string())
    }
}

impl From<SignerError> for SolanaApiError {
    fn from(_: SignerError) -> Self {
        SolanaApiError("Missing required fields".to_string())
    }
}

impl From<bs58::decode::Error> for SolanaApiError {
    fn from(_: bs58::decode::Error) -> Self {
        SolanaApiError("Missing required fields".to_string())
    }
}

impl From<std::str::Utf8Error> for SolanaApiError {
    fn from(_: std::str::Utf8Error) -> Self {
        SolanaApiError("Missing required fields".to_string())
    }
}

impl From<std::io::Error> for SolanaApiError {
    fn from(_: std::io::Error) -> Self {
        SolanaApiError("Missing required fields".to_string())
    }
}

impl From<ProgramError> for SolanaApiError {
    fn from(_: ProgramError) -> Self {
        SolanaApiError("Missing required fields".to_string())
    }
}

impl From<std::string::FromUtf8Error> for SolanaApiError {
    fn from(_: std::string::FromUtf8Error) -> Self {
        SolanaApiError("Missing required fields".to_string())
    }
}

impl From<solana_sdk::pubkey::ParsePubkeyError> for SolanaApiError {
    fn from(_: solana_sdk::pubkey::ParsePubkeyError) -> Self {
        SolanaApiError("Missing required fields".to_string())
    }
}

impl ResponseError for SolanaApiError {
    fn error_response(&self) -> HttpResponse {
        error!("API Error: {}", self.0);
        HttpResponse::BadRequest().json(ServiceErrorResponse {
            success: false,
            error: self.0.clone(),
        })
    }
}
//for this I wrote something different but gpt changed it (fir bhi nahi chala)
impl Responder for SolanaApiError {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        self.error_response().map_into_boxed_body()
    }
}

async fn handle_keypair_generation() -> Result<HttpResponse, SolanaApiError> {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey().to_string();
    let secret = bs58::encode(keypair.to_bytes()).into_string();

    Ok(HttpResponse::Ok().json(ServiceResponse {
        success: true,
        data: KeypairGenResponse { pubkey, secret },
    }))
}

async fn handle_token_creation(req: web::Json<TokenCreateRequest>) -> Result<HttpResponse, SolanaApiError> {
    let mint_authority = Pubkey::from_str(&req.mint_authority)?;
    let mint = Pubkey::from_str(&req.mint)?;

    let instruction = spl_instruction::initialize_mint(
        &spl_token::id(),
        &mint,
        &mint_authority,
        None,
        req.decimals,
    )?;

    let accounts = instruction.accounts.into_iter().map(|acc| InstructionAccountDetails {
        pubkey: acc.pubkey.to_string(),
        is_signer: acc.is_signer,
        is_writable: acc.is_writable,
    }).collect();

    Ok(HttpResponse::Ok().json(ServiceResponse {
        success: true,
        data: InstructionOutput {
            program_id: instruction.program_id.to_string(),
            accounts,
            instruction_data: BASE64_STANDARD.encode(instruction.data),
        },
    }))
}// token creation is right ig, saw from a doc

async fn handle_token_mint(req: web::Json<TokenMintRequest>) -> Result<HttpResponse, SolanaApiError> {
    if req.amount == 0 {
        return Err(SolanaApiError("Missing required fields".to_string()));
    }

    let mint_pubkey = Pubkey::from_str(&req.mint)?;
    let destination_pubkey = Pubkey::from_str(&req.destination)?;
    let authority_pubkey = Pubkey::from_str(&req.authority)?;

    let instruction = spl_instruction::mint_to(
        &spl_token::id(),
        &mint_pubkey,
        &destination_pubkey,
        &authority_pubkey,
        &[],
        req.amount,
    )?;

    let accounts = instruction.accounts.into_iter().map(|acc| InstructionAccountDetails {
        pubkey: acc.pubkey.to_string(),
        is_signer: acc.is_signer,
        is_writable: acc.is_writable,
    }).collect();

    Ok(HttpResponse::Ok().json(ServiceResponse {
        success: true,
        data: InstructionOutput {
            program_id: instruction.program_id.to_string(),
            accounts,
            instruction_data: BASE64_STANDARD.encode(instruction.data),
        },
    }))
}
// I once made the mint token program and used the same code 
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    info!("Solana HTTP server running on http://127.0.0.1:8080");

    HttpServer::new(|| {
        App::new()
            .route("/keypair", web::post().to(handle_keypair_generation))
            .route("/token/create", web::post().to(handle_token_creation))
            .route("/token/mint", web::post().to(handle_token_mint))
    })
    .bind(("0.0.0.0", std::env::var("PORT").unwrap_or("8080".to_string()).parse().unwrap()))?
    .run()
    .await
}
//server bs