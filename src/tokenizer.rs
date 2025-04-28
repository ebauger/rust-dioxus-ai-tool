use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tiktoken_rs::cl100k_base;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TokenEstimator {
    CharDiv4,
    Cl100k,
    Llama2,
    SentencePiece,
}

impl Default for TokenEstimator {
    fn default() -> Self {
        Self::Cl100k
    }
}

impl TokenEstimator {
    pub fn estimate_tokens(&self, text: &str) -> usize {
        match self {
            Self::CharDiv4 => text.chars().count() / 4,
            Self::Cl100k => {
                let bpe = cl100k_base().unwrap();
                bpe.encode_with_special_tokens(text).len()
            }
            Self::Llama2 => {
                // TODO: Implement Llama2 tokenizer when available
                text.chars().count() / 4
            }
            Self::SentencePiece => {
                // TODO: Implement SentencePiece tokenizer when available
                text.chars().count() / 4
            }
        }
    }
}

pub async fn count_tokens(path: &PathBuf, estimator: TokenEstimator) -> std::io::Result<usize> {
    let content = tokio::fs::read_to_string(path).await?;
    Ok(estimator.estimate_tokens(&content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_char_div4_estimator() {
        let estimator = TokenEstimator::CharDiv4;
        assert_eq!(estimator.estimate_tokens("Hello"), 1); // 5 chars / 4
        assert_eq!(estimator.estimate_tokens("Hello World"), 2); // 11 chars / 4
    }

    #[test]
    fn test_cl100k_estimator() {
        let estimator = TokenEstimator::Cl100k;
        let bpe = cl100k_base().unwrap();
        println!(
            "Tokens for 'Hello': {:?}",
            bpe.encode_with_special_tokens("Hello")
        );
        println!(
            "Tokens for 'Hello World': {:?}",
            bpe.encode_with_special_tokens("Hello World")
        );
        assert_eq!(estimator.estimate_tokens("Hello"), 1);
        assert_eq!(estimator.estimate_tokens("Hello World"), 2);
    }

    #[tokio::test]
    async fn test_count_tokens() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, World!";

        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let count = count_tokens(&file_path, TokenEstimator::Cl100k)
            .await
            .unwrap();
        assert_eq!(count, 4); // "Hello", ",", " World", "!"
    }
}
