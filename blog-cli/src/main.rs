use std::{
    fs::{self, File},
    io::{self, Write},
    process::ExitCode,
};

use blog_client::{BlogClient, Transport};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, global(true), help = "use grpc")]
    grpc: bool,
    #[arg(long, global(true), help = "remote server address")]
    server: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    Login {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },
    Get {
        #[arg(long)]
        id: u32,
    },
    Update {
        #[arg(long)]
        id: u32,
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },
    Delete {
        #[arg(long)]
        id: u32,
    },
    List {
        #[arg(long)]
        offset: u32,
        #[arg(long)]
        limit: u8,
    },
}

fn save_token(token: String) -> Result<(), io::Error> {
    let mut f = File::create(".blog_token")?;
    f.write_all(token.as_bytes())
}

fn load_token() -> Result<String, io::Error> {
    fs::read_to_string(".blog_token")
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args = Cli::parse();

    let transport = match args.grpc {
        false => Transport::Http(args.server.unwrap_or("http://localhost:3000".into())),
        true => Transport::Grpc(args.server.unwrap_or("http://localhost:50051".into())),
    };

    let mut blog_client = match BlogClient::new(transport).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("failed to create blog client: {e}");
            return ExitCode::FAILURE;
        }
    };

    match args.command {
        Commands::Login { username, password } => {
            match blog_client.login(username, password).await {
                Ok(log_info) => match save_token(log_info.token) {
                    Ok(_) => {
                        println!("token is saved");
                    }
                    Err(e) => {
                        eprintln!("failed to save token: {e}");
                        return ExitCode::FAILURE;
                    }
                },
                Err(e) => {
                    eprintln!("login error: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        Commands::Register {
            username,
            email,
            password,
        } => match blog_client.register(username, email, password).await {
            Ok(log_info) => match save_token(log_info.token) {
                Ok(_) => {
                    println!("token is saved");
                }
                Err(e) => {
                    eprintln!("Failed to save token: {e}");
                    return ExitCode::FAILURE;
                }
            },
            Err(e) => {
                eprintln!("register error: {e}");
                return ExitCode::FAILURE;
            }
        },
        Commands::Create { title, content } => {
            let token = match load_token() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Не удалось загрузить токен: {e}");
                    return ExitCode::FAILURE;
                }
            };
            match blog_client.create_post(title, content, token).await {
                Ok(post) => {
                    println!("Пост создан: {post:?}");
                }
                Err(e) => {
                    eprintln!("Ошибка при создании поста: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        Commands::Update { id, title, content } => {
            let token = match load_token() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Не удалось загрузить токен: {e}");
                    return ExitCode::FAILURE;
                }
            };
            match blog_client.update_post(id, title, content, token).await {
                Ok(post) => {
                    println!("Пост обновленю. {post:?}");
                }
                Err(e) => {
                    eprintln!("Ошибка при обновлении поста: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        Commands::Get { id } => match blog_client.get_post(id).await {
            Ok(post) => {
                println!("Пост. {post:?}");
            }
            Err(e) => {
                eprintln!("Ошибка при получении поста: {e}");
                return ExitCode::FAILURE;
            }
        },
        Commands::Delete { id } => {
            let token = match load_token() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Не удалось загрузить токен: {e}");
                    return ExitCode::FAILURE;
                }
            };
            match blog_client.delete_post(id, token).await {
                Ok(_) => {
                    println!("Пост удален");
                }
                Err(e) => {
                    eprintln!("Не удалось удалить пост: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        Commands::List { offset, limit } => match blog_client.list_posts(limit, offset).await {
            Ok(r) => {
                println!("Список постов: offset {} limit {}", r.offset, r.limit);
                r.posts.iter().for_each(|p| {
                    println!("{p:?}");
                });
            }
            Err(e) => {
                eprintln!("Не удалось получить список постов: {e}");
                return ExitCode::FAILURE;
            }
        },
    };
    ExitCode::SUCCESS
}
