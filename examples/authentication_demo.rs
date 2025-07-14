//! Demo of 6-digit verification code authentication system for CrossCopy
//! 
//! This example demonstrates the authentication flow between two devices
//! using a 6-digit verification code for secure device pairing.

use crosscopy::config::NetworkConfig;
use crosscopy::events::EventBus;
use crosscopy::network::NetworkManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio::io::{self, AsyncBufReadExt, BufReader};

// Mock authentication structures for demonstration
#[derive(Debug, Clone)]
pub struct AuthChallenge {
    pub challenge_id: String,
    pub verification_code: String,
    pub expires_at: u64,
    pub device_name: String,
}

#[derive(Debug, Clone)]
pub struct AuthResponse {
    pub challenge_id: String,
    pub verification_code: String,
}

#[derive(Debug, Clone)]
pub struct AuthResult {
    pub success: bool,
    pub error_message: Option<String>,
}

pub struct AuthenticationDemo {
    device_name: String,
    is_server: bool,
    active_challenge: Option<AuthChallenge>,
}

impl AuthenticationDemo {
    pub fn new(device_name: String, is_server: bool) -> Self {
        Self {
            device_name,
            is_server,
            active_challenge: None,
        }
    }

    /// Generate a 6-digit verification code
    pub fn generate_verification_code() -> String {
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        format!("{:06}", rng.gen_range(100000..=999999))
    }

    /// Create an authentication challenge (server side)
    pub fn create_challenge(&mut self, requesting_device: &str) -> AuthChallenge {
        let challenge_id = uuid::Uuid::new_v4().to_string();
        let verification_code = Self::generate_verification_code();
        let expires_at = chrono::Utc::now().timestamp() as u64 + 300; // 5 minutes

        let challenge = AuthChallenge {
            challenge_id,
            verification_code: verification_code.clone(),
            expires_at,
            device_name: requesting_device.to_string(),
        };

        self.active_challenge = Some(challenge.clone());

        println!("\n🔐 Authentication Challenge Created");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📱 Device '{}' wants to connect", requesting_device);
        println!("🔢 Verification Code: {}", verification_code);
        println!("⏰ Expires in: 5 minutes");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Please share this code with the requesting device.");

        challenge
    }

    /// Verify the authentication response (server side)
    pub fn verify_response(&mut self, response: &AuthResponse) -> AuthResult {
        if let Some(ref challenge) = self.active_challenge {
            // Check if challenge ID matches
            if challenge.challenge_id != response.challenge_id {
                return AuthResult {
                    success: false,
                    error_message: Some("Invalid challenge ID".to_string()),
                };
            }

            // Check if not expired
            let now = chrono::Utc::now().timestamp() as u64;
            if now > challenge.expires_at {
                self.active_challenge = None;
                return AuthResult {
                    success: false,
                    error_message: Some("Verification code expired".to_string()),
                };
            }

            // Check verification code
            if challenge.verification_code == response.verification_code {
                self.active_challenge = None;
                println!("\n✅ Authentication Successful!");
                println!("Device '{}' has been authenticated and trusted.", challenge.device_name);
                
                AuthResult {
                    success: true,
                    error_message: None,
                }
            } else {
                println!("\n❌ Authentication Failed!");
                println!("Incorrect verification code entered.");
                
                AuthResult {
                    success: false,
                    error_message: Some("Incorrect verification code".to_string()),
                }
            }
        } else {
            AuthResult {
                success: false,
                error_message: Some("No active challenge".to_string()),
            }
        }
    }

    /// Simulate client-side authentication flow
    pub async fn simulate_client_authentication(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔗 Initiating connection to '{}'...", "Server Device");
        sleep(Duration::from_millis(500)).await;

        println!("✅ Connection established");
        println!("🔐 Waiting for authentication challenge...");
        sleep(Duration::from_millis(1000)).await;

        // Simulate receiving challenge
        let challenge = AuthChallenge {
            challenge_id: uuid::Uuid::new_v4().to_string(),
            verification_code: "123456".to_string(),
            expires_at: chrono::Utc::now().timestamp() as u64 + 300,
            device_name: "Server Device".to_string(),
        };

        println!("\n📨 Authentication challenge received from '{}'", challenge.device_name);
        println!("Please enter the 6-digit verification code shown on the server device:");

        // Get user input
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut input = String::new();
        
        print!("Enter code: ");
        io::Write::flush(&mut io::stdout())?;
        reader.read_line(&mut input).await?;
        
        let user_code = input.trim().to_string();

        // Create response
        let response = AuthResponse {
            challenge_id: challenge.challenge_id,
            verification_code: user_code,
        };

        println!("📤 Sending authentication response...");
        sleep(Duration::from_millis(500)).await;

        // Simulate verification result
        if response.verification_code == "123456" {
            println!("✅ Authentication successful! You are now connected.");
        } else {
            println!("❌ Authentication failed! Incorrect verification code.");
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("CrossCopy Authentication Demo");
    println!("============================");
    
    // Get user choice
    println!("Choose mode:");
    println!("1. Server (generates verification code)");
    println!("2. Client (enters verification code)");
    print!("Enter choice (1 or 2): ");
    io::Write::flush(&mut io::stdout())?;
    
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut input = String::new();
    reader.read_line(&mut input).await?;
    
    let choice = input.trim();
    
    match choice {
        "1" => {
            // Server mode
            println!("\n🖥️  Running in SERVER mode");
            let mut auth_demo = AuthenticationDemo::new("My Server Device".to_string(), true);
            
            // Simulate incoming connection
            println!("📡 Waiting for incoming connections...");
            sleep(Duration::from_secs(2)).await;
            
            // Create challenge
            let challenge = auth_demo.create_challenge("Client Device");
            
            // Wait for user response simulation
            println!("\nWaiting for client response...");
            sleep(Duration::from_secs(3)).await;
            
            // Simulate correct response
            let response = AuthResponse {
                challenge_id: challenge.challenge_id,
                verification_code: challenge.verification_code,
            };
            
            let result = auth_demo.verify_response(&response);
            
            if result.success {
                println!("\n🎉 Device pairing completed successfully!");
                println!("The devices can now sync clipboard content securely.");
            } else {
                println!("\n💥 Device pairing failed: {}", 
                    result.error_message.unwrap_or("Unknown error".to_string()));
            }
        },
        "2" => {
            // Client mode
            println!("\n📱 Running in CLIENT mode");
            let auth_demo = AuthenticationDemo::new("My Client Device".to_string(), false);
            
            auth_demo.simulate_client_authentication().await?;
        },
        _ => {
            println!("Invalid choice. Please run the demo again and select 1 or 2.");
        }
    }
    
    println!("\n📋 Demo completed. In a real implementation:");
    println!("   • The verification code would be displayed in the UI");
    println!("   • Network communication would use libp2p messages");
    println!("   • Device trust would be persisted to configuration");
    println!("   • Multiple security layers would be active");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_code_generation() {
        let code = AuthenticationDemo::generate_verification_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        
        // Test that codes are different
        let code2 = AuthenticationDemo::generate_verification_code();
        // Note: There's a tiny chance they could be the same, but very unlikely
        println!("Generated codes: {} and {}", code, code2);
    }

    #[test]
    fn test_challenge_creation() {
        let mut auth_demo = AuthenticationDemo::new("Test Server".to_string(), true);
        let challenge = auth_demo.create_challenge("Test Client");
        
        assert!(!challenge.challenge_id.is_empty());
        assert_eq!(challenge.verification_code.len(), 6);
        assert!(challenge.expires_at > chrono::Utc::now().timestamp() as u64);
        assert_eq!(challenge.device_name, "Test Client");
    }

    #[test]
    fn test_successful_verification() {
        let mut auth_demo = AuthenticationDemo::new("Test Server".to_string(), true);
        let challenge = auth_demo.create_challenge("Test Client");
        
        let response = AuthResponse {
            challenge_id: challenge.challenge_id,
            verification_code: challenge.verification_code,
        };
        
        let result = auth_demo.verify_response(&response);
        assert!(result.success);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_failed_verification() {
        let mut auth_demo = AuthenticationDemo::new("Test Server".to_string(), true);
        let challenge = auth_demo.create_challenge("Test Client");
        
        let response = AuthResponse {
            challenge_id: challenge.challenge_id,
            verification_code: "wrong_code".to_string(),
        };
        
        let result = auth_demo.verify_response(&response);
        assert!(!result.success);
        assert!(result.error_message.is_some());
    }
}
