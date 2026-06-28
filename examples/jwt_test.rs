use tezweb::auth::{jwt_encode, jwt_decode};

fn main() {
    let secret = "my-secret-key";

    // Encode a token for user "akram", valid for 60 seconds
    let token = jwt_encode("akram", secret, 60);
    println!("✅ Token generated: {}", token);

    // Decode and verify it
    match jwt_decode(&token, secret) {
        Some(claims) => println!("✅ Token valid! sub={}, exp={}", claims.sub, claims.exp),
        None => println!("❌ Token invalid"),
    }

    // Try with wrong secret — should fail
    match jwt_decode(&token, "wrong-secret") {
        Some(_) => println!("❌ SECURITY BUG: wrong secret accepted!"),
        None => println!("✅ Correctly rejected wrong secret"),
    }

    // Try with tampered token — should fail
    let tampered = format!("{}x", token);
    match jwt_decode(&tampered, secret) {
        Some(_) => println!("❌ SECURITY BUG: tampered token accepted!"),
        None => println!("✅ Correctly rejected tampered token"),
    }
}
