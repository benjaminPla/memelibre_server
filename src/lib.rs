use aws_config::SdkConfig;
use aws_credential_types::{provider::SharedCredentialsProvider, Credentials};
use aws_sdk_s3::{
    config::{BehaviorVersion, Region},
    Client,
};
use axum::http::StatusCode;
use rand::rng;
use rand::seq::IndexedRandom;

mod macros;
mod models;

pub async fn create_bucket_client() -> Result<Client, String> {
    let config = models::Config::from_env().expect("Error creating Config");

    let credentials = Credentials::new(
        config.bucket_key,
        config.bucket_secret,
        None,
        None,
        "digitalocean",
    );

    let credentials_provider = SharedCredentialsProvider::new(credentials);

    let sdk_config = SdkConfig::builder()
        .region(Some(Region::new(config.bucket_region)))
        .endpoint_url(config.bucket_endpoint)
        .credentials_provider(credentials_provider)
        .behavior_version(BehaviorVersion::latest())
        .build();

    Ok(Client::new(&sdk_config))
}

pub fn generate_username() -> Result<String, (StatusCode, String)> {
    let adjectives = [
        "acelerado",
        "ajustado",
        "anarcocapitalista",
        "anti-casta",
        "antikeynesiano",
        "antisistema",
        "antisocialista",
        "austero",
        "austriaco",
        "autónomo",
        "autosuficiente",
        "basado",
        "capitalista",
        "coherente",
        "competitivo",
        "crudo",
        "desatado",
        "despertado",
        "despierto",
        "dolarizado",
        "eficiente",
        "enojado",
        "épico",
        "estético",
        "explosivo",
        "filoso",
        "finito",
        "genuino",
        "iconoclasta",
        "iluminado",
        "imparable",
        "incendiario",
        "incorregible",
        "individualista",
        "inflamable",
        "inflexible",
        "insoportable",
        "intenso",
        "leonino",
        "letal",
        "libertario",
        "marginal",
        "mercadolibre",
        "motosierra",
        "motosierrado",
        "ortodoxo",
        "peligroso",
        "privatizado",
        "profamilia",
        "promercado",
        "provida",
        "racional",
        "rebelde",
        "recortado",
        "resistente",
        "rupturista",
        "sincero",
        "sinEstado",
        "sinMinisterios",
        "sinplaneros",
        "soberano",
        "sustentable",
        "ultraliberal",
        "viral",
    ];

    let nouns = [
        "abismo",
        "águila",
        "ajuste",
        "anarcocapitalismo",
        "avatar",
        "biblia",
        "bigote",
        "billete",
        "búho",
        "cadena",
        "calle",
        "caos",
        "casta",
        "caverna",
        "código",
        "congreso",
        "constitución",
        "corte",
        "cuervo",
        "déficit",
        "desierto",
        "diputado",
        "discurso",
        "dólar",
        "dragón",
        "drone",
        "eco",
        "espejismo",
        "estado",
        "fénix",
        "gráfico",
        "hiena",
        "humo",
        "impuesto",
        "inflación",
        "katana",
        "kraken",
        "león",
        "libertad",
        "libro",
        "llama",
        "memazo",
        "meme",
        "mercado",
        "micrófono",
        "milagro",
        "militante",
        "ministerio",
        "mono",
        "montaña",
        "motosierra",
        "perfil",
        "pesos",
        "piquetero",
        "planero",
        "póster",
        "privatización",
        "puma",
        "puñal",
        "rayo",
        "regulación",
        "relámpago",
        "rinoceronte",
        "satélite",
        "serpiente",
        "silla",
        "subsidio",
        "tanque",
        "teclado",
        "tormenta",
        "traje",
        "trueno",
        "tuit",
        "urna",
        "volcán",
        "voto",
    ];

    let mut rng = rng();
    let adjective = adjectives
        .choose(&mut rng)
        .ok_or_else(|| http_error!(StatusCode::INTERNAL_SERVER_ERROR))?;
    let noun = nouns
        .choose(&mut rng)
        .ok_or_else(|| http_error!(StatusCode::INTERNAL_SERVER_ERROR))?;
    let number = rand::random::<u16>() % 1000;

    Ok(format!("{}_{}_{}", adjective, noun, number))
}
