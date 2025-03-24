
#[cfg(test)]
mod integration_tests {
    use std::process::{Command, Child};
    use std::thread::sleep;
    use std::time::Duration;
    use std::io::{BufRead, BufReader};
    use std::fs::File;
    use std::path::Path;

    #[test]
    fn test_server_client_interaction() {
        // Start the server
        let server = start_server();
        
        // Wait for server to initialize
        sleep(Duration::from_secs(3));
        
        // Run the client and capture output
        let client_output = run_client();
        
        // Clean up
        stop_server(server);
        
        // Check output for success indicators
        assert!(client_output.contains("Created invoice:"));
        assert!(client_output.contains("Status: Pending"));
    }
    
    fn start_server() -> Child {
        Command::new("cargo")
            .args(["run", "--bin", "btc-pay-server"])
            .spawn()
            .expect("Failed to start server")
    }
    
    fn run_client() -> String {
        let output = Command::new("cargo")
            .args(["run", "--bin", "client"])
            .output()
            .expect("Failed to execute client");
            
        String::from_utf8_lossy(&output.stdout).to_string()
    }
    
    fn stop_server(mut server: Child) {
        server.kill().expect("Failed to kill server process");
    }
}
