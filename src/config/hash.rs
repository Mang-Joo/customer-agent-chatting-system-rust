use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use tracing::error;

use super::{error::AppError, MangJooResult};

pub async fn hash(plain_data: &String) -> MangJooResult<String> {
    // 랜덤 솔트 생성
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 인스턴스 생성 (기본 파라미터 사용)
    let argon2 = Argon2::default();

    // 비밀번호 해싱 - 수정된 부분
    let password_hash = argon2.hash_password(plain_data.as_bytes(), salt.as_salt());

    let hash_data = password_hash
        .map(|hash| hash)
        .map_err(|err| AppError::InternalError(format!("Hashing Error {}", err.to_string())))?;

    Ok(hash_data.to_string())
}

pub async fn verify(plain_data: &String, hash_data: &String) -> bool {
    let argon2 = Argon2::default();

    let hasher = PasswordHash::new(&hash_data);
    let hash = match hasher {
        Ok(hash) => hash,
        Err(err) => {
            error!("Verify Password Error");
            eprintln!("Verify Password Error :  {:?}", err);
            return false;
        }
    };

    argon2.verify_password(plain_data.as_bytes(), &hash).is_ok()
}
