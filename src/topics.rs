//! Topic tags for lectures, from keywords in their names and descriptions.
//!
//! Titles are tagged in English keys ("Salah"); the UI translates the keys.
//! Keywords cover English, Bangla and the usual transliterations, because
//! the same talk may be titled "Namaz", "Salah", "Prayer" or "নামাজ".

/// (topic key, keywords). ASCII keywords match whole words; Bangla and
/// Arabic keywords match anywhere (they are written without spaces as
/// often as with).
pub const TOPICS: &[(&str, &[&str])] = &[
    ("Quran", &["quran", "qur'an", "koran", "tilawat", "tilawah", "recitation", "কুরআন", "কোরআন", "তিলাওয়াত", "قرآن"]),
    ("Tafsir", &["tafsir", "tafseer", "tafsir-ul", "তাফসীর", "তাফসির", "تفسير"]),
    ("Hadith", &["hadith", "hadees", "hadis", "sunnah", "bukhari", "muslim sharif", "হাদিস", "হাদীস", "সুন্নাহ", "حديث"]),
    ("Seerah", &["seerah", "sirah", "seerat", "prophet muhammad", "life of the prophet", "rasulullah", "সীরাত", "সিরাত", "নবীজি", "রাসূল", "রাসুল", "سيرة"]),
    ("Prophets", &["prophets", "stories of the prophets", "qasas", "qasas al-anbiya", "anbiya", "নবীদের", "নবী", "ইউসুফ", "মুসা", "ইব্রাহিম"]),
    ("Sahaba", &["sahaba", "sahabah", "companions", "abu bakr", "umar ibn", "uthman ibn", "ali ibn", "সাহাবী", "সাহাবা", "সাহাবায়ে"]),
    ("Aqeedah", &["aqeedah", "aqidah", "tawheed", "tawhid", "iman", "imaan", "shirk", "belief", "faith", "আকিদা", "আকীদা", "তাওহীদ", "তাওহিদ", "ঈমান", "শিরক"]),
    ("Salah", &["salah", "salat", "namaz", "namaaz", "prayer", "prayers", "jummah", "jumuah", "tahajjud", "wudu", "নামাজ", "নামায", "সালাত", "জুমা", "তাহাজ্জুদ", "অজু", "ওযু", "صلاة"]),
    ("Ramadan", &["ramadan", "ramzan", "ramadhan", "fasting", "sawm", "roza", "iftar", "suhoor", "sehri", "laylatul qadr", "itikaf", "রমজান", "রমাদান", "রোজা", "রোযা", "সিয়াম", "ইফতার", "সেহরি", "শবে কদর", "رمضان"]),
    ("Hajj", &["hajj", "umrah", "umra", "makkah", "mecca", "হজ", "হজ্জ", "ওমরা", "উমরা", "মক্কা"]),
    ("Zakat", &["zakat", "zakah", "sadaqah", "sadaqa", "charity", "যাকাত", "জাকাত", "সদকা", "দান"]),
    ("Dua", &["dua", "duaa", "dhikr", "zikr", "azkar", "adhkar", "istighfar", "দোয়া", "দুআ", "দোআ", "জিকির", "যিকির", "ইস্তিগফার", "دعاء"]),
    ("Family", &["family", "marriage", "nikah", "husband", "wife", "spouse", "divorce", "পরিবার", "বিয়ে", "বিবাহ", "স্বামী", "স্ত্রী", "দাম্পত্য"]),
    ("Parenting", &["parenting", "parents", "children", "child", "tarbiyah", "upbringing", "সন্তান", "বাবা মা", "মা বাবা", "পিতা মাতা", "শিশু"]),
    ("Youth", &["youth", "young", "teen", "teenagers", "students", "যুবক", "তরুণ", "যুবসমাজ", "ছাত্র"]),
    ("Women", &["women", "woman", "sisters", "hijab", "niqab", "নারী", "মহিলা", "বোন", "পর্দা", "হিজাব"]),
    ("Akhirah", &["akhirah", "akhira", "hereafter", "death", "grave", "qabr", "judgement day", "qiyamah", "afterlife", "আখিরাত", "আখেরাত", "মৃত্যু", "কবর", "কিয়ামত", "কেয়ামত", "হাশর"]),
    ("Jannah", &["jannah", "paradise", "jahannam", "hellfire", "hell", "জান্নাত", "জাহান্নাম", "বেহেশত", "দোজখ"]),
    ("Character", &["akhlaq", "character", "manners", "adab", "honesty", "anger", "pride", "arrogance", "backbiting", "আখলাক", "চরিত্র", "আদব", "অহংকার", "গীবত", "রাগ"]),
    ("Patience", &["patience", "sabr", "gratitude", "shukr", "hardship", "trials", "tests of life", "ধৈর্য", "সবর", "শুকর", "কৃতজ্ঞতা", "বিপদ"]),
    ("Tawbah", &["tawbah", "tawba", "repentance", "forgiveness", "sins", "তওবা", "তাওবা", "ক্ষমা", "গুনাহ", "পাপ"]),
    ("Fiqh", &["fiqh", "masala", "masail", "halal", "haram", "ruling", "rulings", "ফিকহ", "মাসআলা", "মাসায়েল", "হালাল", "হারাম"]),
    ("Business", &["business", "trade", "riba", "interest", "earning", "rizq", "money", "finance", "ব্যবসা", "সুদ", "রিজিক", "উপার্জন", "অর্থনীতি"]),
    ("History", &["history", "khilafah", "caliphate", "ottoman", "andalus", "battle", "ইতিহাস", "খিলাফত", "যুদ্ধ"]),
    ("Science", &["science", "scientific", "miracles of the quran", "universe", "বিজ্ঞান", "মহাবিশ্ব"]),
    ("Dawah", &["dawah", "da'wah", "comparative religion", "debate", "atheism", "christianity", "দাওয়াহ", "দাওয়াত", "নাস্তিক"]),
    ("Kids", &["kids", "for children", "cartoon", "animated", "bedtime", "শিশুদের", "বাচ্চাদের", "কার্টুন"]),
    ("Nasheed", &["nasheed", "nasheeds", "anasheed", "naat", "naats", "hamd", "gojol", "ghazal", "qasida", "নাশিদ", "গজল", "হামদ", "নাত", "কাসিদা"]),
    ("Waz", &["waz", "waaz", "mahfil", "bayan", "khutbah", "khutba", "sermon", "ওয়াজ", "মাহফিল", "বয়ান", "খুতবা"]),
    ("Arabic", &["learn arabic", "arabic grammar", "arabic language", "nahw", "sarf", "আরবি ভাষা", "আরবী"]),
];

/// Topic keys whose keywords appear in `text`.
pub fn detect(text: &str) -> Vec<&'static str> {
    let lower = text.to_lowercase();
    TOPICS
        .iter()
        .filter(|(_, words)| words.iter().any(|w| contains_word(&lower, w)))
        .map(|(topic, _)| *topic)
        .collect()
}

/// Whole-word match for ASCII keywords (so "isa" doesn't match "visa"),
/// plain substring match for other scripts.
fn contains_word(haystack: &str, word: &str) -> bool {
    if !word.is_ascii() {
        return haystack.contains(word);
    }
    let bytes = haystack.as_bytes();
    let mut from = 0;
    while let Some(i) = haystack[from..].find(word) {
        let start = from + i;
        let end = start + word.len();
        let before_ok = start == 0 || !bytes[start - 1].is_ascii_alphanumeric();
        let after_ok = end == bytes.len() || !bytes[end].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

/// Topics as stored on a title: ",Salah,Ramadan," (empty string for none).
pub fn to_column(topics: &[&str]) -> Option<String> {
    (!topics.is_empty()).then(|| format!(",{},", topics.join(",")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_in_both_languages() {
        assert_eq!(detect("The Night Prayer (Tahajjud)"), vec!["Salah"]);
        assert_eq!(detect("রমজানের রোজা ও তাকওয়া"), vec!["Ramadan"]);
        let t = detect("Stories of the Prophets - Yusuf");
        assert!(!detect("Yusuf Estes - Q&A").contains(&"Prophets"));
        assert!(t.contains(&"Prophets"));
        assert!(detect("visa application").is_empty(), "'isa' must not match inside 'visa'");
        assert_eq!(to_column(&["Salah", "Dua"]).as_deref(), Some(",Salah,Dua,"));
        assert_eq!(to_column(&[]), None);
    }
}
