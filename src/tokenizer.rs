use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use tiktoken_rs::cl100k_base;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenEstimator {
    CharDiv4,
    Cl100k,
    Llama2,
    SentencePiece,
}

impl fmt::Display for TokenEstimator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CharDiv4 => write!(f, "CharDiv4"),
            Self::Cl100k => write!(f, "Cl100k"),
            Self::Llama2 => write!(f, "Llama2"),
            Self::SentencePiece => write!(f, "SentencePiece"),
        }
    }
}

impl FromStr for TokenEstimator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CharDiv4" => Ok(Self::CharDiv4),
            "Cl100k" => Ok(Self::Cl100k),
            "Llama2" => Ok(Self::Llama2),
            "SentencePiece" => Ok(Self::SentencePiece),
            _ => Err(format!("Unknown token estimator: {}", s)),
        }
    }
}

impl Default for TokenEstimator {
    fn default() -> Self {
        Self::CharDiv4
    }
}

impl TokenEstimator {
    pub fn name(&self) -> &'static str {
        match self {
            Self::CharDiv4 => "Char/4 heuristic",
            Self::Cl100k => "GPT-3/4 (cl100k)",
            Self::Llama2 => "Llama2 BPE",
            Self::SentencePiece => "Gemini SentencePiece",
        }
    }

    pub fn estimate_tokens(&self, text: &str) -> usize {
        match self {
            Self::CharDiv4 => text.chars().count() / 4,
            Self::Cl100k => {
                let bpe = cl100k_base().unwrap();
                bpe.encode_with_special_tokens(text).len()
            }
            Self::Llama2 => {
                // TODO: Implement Llama2 tokenizer
                text.chars().count() / 4 // Fallback for now
            }
            Self::SentencePiece => {
                // TODO: Implement SentencePiece tokenizer
                text.chars().count() / 4 // Fallback for now
            }
        }
    }

    pub fn estimate_file_tokens(&self, path: &Path) -> std::io::Result<usize> {
        let content = std::fs::read_to_string(path)?;
        Ok(self.estimate_tokens(&content))
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
    fn test_char_div4() {
        let estimator = TokenEstimator::CharDiv4;
        assert_eq!(estimator.estimate_tokens("Hello"), 1); // 5 chars / 4 = 1
        assert_eq!(estimator.estimate_tokens("Hello World"), 2); // 11 chars / 4 = 2
    }

    #[test]
    fn test_cl100k() {
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
