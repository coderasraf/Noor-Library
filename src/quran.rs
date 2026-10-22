//! The 114 surahs of the Quran, for naming recitation files ("018.mp3").
//!
//! Names, meanings, verse counts and place of revelation come from the
//! Quran.com API (api.quran.com/api/v4/chapters, English and Bangla), so
//! they match a widely used reference rather than hand-typed spellings.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Surah {
    pub number: u32,
    /// Transliterated name, e.g. "Al-Kahf".
    pub name: &'static str,
    pub arabic: &'static str,
    pub meaning_en: &'static str,
    pub meaning_bn: &'static str,
    pub verses: u32,
    /// Revealed in Makkah (true) or Madinah (false).
    pub makki: bool,
}

pub fn surah(number: u32) -> Option<&'static Surah> {
    number
        .checked_sub(1)
        .and_then(|i| SURAHS.get(i as usize))
}

pub const SURAHS: [Surah; 114] = [
    Surah { number: 1, name: "Al-Fatihah", arabic: "الفاتحة", meaning_en: "The Opener", meaning_bn: "সূচনা", verses: 7, makki: true },
    Surah { number: 2, name: "Al-Baqarah", arabic: "البقرة", meaning_en: "The Cow", meaning_bn: "বকনা-বাছুর", verses: 286, makki: false },
    Surah { number: 3, name: "Ali 'Imran", arabic: "آل عمران", meaning_en: "Family of Imran", meaning_bn: "ইমরানের পরিবার", verses: 200, makki: false },
    Surah { number: 4, name: "An-Nisa", arabic: "النساء", meaning_en: "The Women", meaning_bn: "নারী", verses: 176, makki: false },
    Surah { number: 5, name: "Al-Ma'idah", arabic: "المائدة", meaning_en: "The Table Spread", meaning_bn: "খাদ্য পরিবেশিত টেবিল", verses: 120, makki: false },
    Surah { number: 6, name: "Al-An'am", arabic: "الأنعام", meaning_en: "The Cattle", meaning_bn: "গৃহপালিত পশু", verses: 165, makki: true },
    Surah { number: 7, name: "Al-A'raf", arabic: "الأعراف", meaning_en: "The Heights", meaning_bn: "উচু স্থান", verses: 206, makki: true },
    Surah { number: 8, name: "Al-Anfal", arabic: "الأنفال", meaning_en: "The Spoils of War", meaning_bn: "যুদ্ধ-লব্ধ ধনসম্পদ", verses: 75, makki: false },
    Surah { number: 9, name: "At-Tawbah", arabic: "التوبة", meaning_en: "The Repentance", meaning_bn: "অনুশোচনা", verses: 129, makki: false },
    Surah { number: 10, name: "Yunus", arabic: "يونس", meaning_en: "Jonah", meaning_bn: "নবী ইউনুস", verses: 109, makki: true },
    Surah { number: 11, name: "Hud", arabic: "هود", meaning_en: "Hud", meaning_bn: "নবী হুদ", verses: 123, makki: true },
    Surah { number: 12, name: "Yusuf", arabic: "يوسف", meaning_en: "Joseph", meaning_bn: "নবী ইউসুফ", verses: 111, makki: true },
    Surah { number: 13, name: "Ar-Ra'd", arabic: "الرعد", meaning_en: "The Thunder", meaning_bn: "বজ্রপাত", verses: 43, makki: false },
    Surah { number: 14, name: "Ibrahim", arabic: "ابراهيم", meaning_en: "Abraham", meaning_bn: "নবী ইবরাহীম", verses: 52, makki: true },
    Surah { number: 15, name: "Al-Hijr", arabic: "الحجر", meaning_en: "The Rocky Tract", meaning_bn: "পাথুরে পাহাড়", verses: 99, makki: true },
    Surah { number: 16, name: "An-Nahl", arabic: "النحل", meaning_en: "The Bee", meaning_bn: "মৌমাছি", verses: 128, makki: true },
    Surah { number: 17, name: "Al-Isra", arabic: "الإسراء", meaning_en: "The Night Journey", meaning_bn: "ইহুদি জাতি", verses: 111, makki: true },
    Surah { number: 18, name: "Al-Kahf", arabic: "الكهف", meaning_en: "The Cave", meaning_bn: "গুহা", verses: 110, makki: true },
    Surah { number: 19, name: "Maryam", arabic: "مريم", meaning_en: "Mary", meaning_bn: "মারইয়াম (ঈসা নবীর মা)", verses: 98, makki: true },
    Surah { number: 20, name: "Taha", arabic: "طه", meaning_en: "Ta-Ha", meaning_bn: "ত়া হা", verses: 135, makki: true },
    Surah { number: 21, name: "Al-Anbya", arabic: "الأنبياء", meaning_en: "The Prophets", meaning_bn: "নবীগণ", verses: 112, makki: true },
    Surah { number: 22, name: "Al-Hajj", arabic: "الحج", meaning_en: "The Pilgrimage", meaning_bn: "হ়াজ্জ", verses: 78, makki: false },
    Surah { number: 23, name: "Al-Mu'minun", arabic: "المؤمنون", meaning_en: "The Believers", meaning_bn: "বিশ্বাসী", verses: 118, makki: true },
    Surah { number: 24, name: "An-Nur", arabic: "النور", meaning_en: "The Light", meaning_bn: "আলো", verses: 64, makki: false },
    Surah { number: 25, name: "Al-Furqan", arabic: "الفرقان", meaning_en: "The Criterion", meaning_bn: "মানদণ্ড", verses: 77, makki: true },
    Surah { number: 26, name: "Ash-Shu'ara", arabic: "الشعراء", meaning_en: "The Poets", meaning_bn: "কবি", verses: 227, makki: true },
    Surah { number: 27, name: "An-Naml", arabic: "النمل", meaning_en: "The Ant", meaning_bn: "পিঁপড়া", verses: 93, makki: true },
    Surah { number: 28, name: "Al-Qasas", arabic: "القصص", meaning_en: "The Stories", meaning_bn: "কাহিনি", verses: 88, makki: true },
    Surah { number: 29, name: "Al-'Ankabut", arabic: "العنكبوت", meaning_en: "The Spider", meaning_bn: "মাকড়শা", verses: 69, makki: true },
    Surah { number: 30, name: "Ar-Rum", arabic: "الروم", meaning_en: "The Romans", meaning_bn: "রোমান জাতি", verses: 60, makki: true },
    Surah { number: 31, name: "Luqman", arabic: "لقمان", meaning_en: "Luqman", meaning_bn: "এক জ্ঞানী ব্যাক্তি", verses: 34, makki: true },
    Surah { number: 32, name: "As-Sajdah", arabic: "السجدة", meaning_en: "The Prostration", meaning_bn: "সিজদা", verses: 30, makki: true },
    Surah { number: 33, name: "Al-Ahzab", arabic: "الأحزاب", meaning_en: "The Combined Forces", meaning_bn: "জোট", verses: 73, makki: false },
    Surah { number: 34, name: "Saba", arabic: "سبإ", meaning_en: "Sheba", meaning_bn: "রানী সাবা/শেবা", verses: 54, makki: true },
    Surah { number: 35, name: "Fatir", arabic: "فاطر", meaning_en: "Originator", meaning_bn: "আদি স্রষ্টা", verses: 45, makki: true },
    Surah { number: 36, name: "Ya-Sin", arabic: "يس", meaning_en: "Ya Sin", meaning_bn: "ইয়াসীন", verses: 83, makki: true },
    Surah { number: 37, name: "As-Saffat", arabic: "الصافات", meaning_en: "Those who set the Ranks", meaning_bn: "সারিবদ্ধভাবে দাঁড়ানো", verses: 182, makki: true },
    Surah { number: 38, name: "Sad", arabic: "ص", meaning_en: "The Letter \"Saad\"", meaning_bn: "আরবি বর্ণ সাদ", verses: 88, makki: true },
    Surah { number: 39, name: "Az-Zumar", arabic: "الزمر", meaning_en: "The Troops", meaning_bn: "দলবদ্ধ জনতা", verses: 75, makki: true },
    Surah { number: 40, name: "Ghafir", arabic: "غافر", meaning_en: "The Forgiver", meaning_bn: "ক্ষমাকারী", verses: 85, makki: true },
    Surah { number: 41, name: "Fussilat", arabic: "فصلت", meaning_en: "Explained in Detail", meaning_bn: "সুস্পষ্ট বিবরণ", verses: 54, makki: true },
    Surah { number: 42, name: "Ash-Shuraa", arabic: "الشورى", meaning_en: "The Consultation", meaning_bn: "পরামর্শ", verses: 53, makki: true },
    Surah { number: 43, name: "Az-Zukhruf", arabic: "الزخرف", meaning_en: "The Ornaments of Gold", meaning_bn: "সোনাদানা", verses: 89, makki: true },
    Surah { number: 44, name: "Ad-Dukhan", arabic: "الدخان", meaning_en: "The Smoke", meaning_bn: "ধোঁয়া", verses: 59, makki: true },
    Surah { number: 45, name: "Al-Jathiyah", arabic: "الجاثية", meaning_en: "The Crouching", meaning_bn: "নতজানু", verses: 37, makki: true },
    Surah { number: 46, name: "Al-Ahqaf", arabic: "الأحقاف", meaning_en: "The Wind-Curved Sandhills", meaning_bn: "বালুর পাহাড়", verses: 35, makki: true },
    Surah { number: 47, name: "Muhammad", arabic: "محمد", meaning_en: "Muhammad", meaning_bn: "নবী মুহাম্মদ", verses: 38, makki: false },
    Surah { number: 48, name: "Al-Fath", arabic: "الفتح", meaning_en: "The Victory", meaning_bn: "বিজয়", verses: 29, makki: false },
    Surah { number: 49, name: "Al-Hujurat", arabic: "الحجرات", meaning_en: "The Rooms", meaning_bn: "আবাস", verses: 18, makki: false },
    Surah { number: 50, name: "Qaf", arabic: "ق", meaning_en: "The Letter \"Qaf\"", meaning_bn: "কাফ", verses: 45, makki: true },
    Surah { number: 51, name: "Adh-Dhariyat", arabic: "الذاريات", meaning_en: "The Winnowing Winds", meaning_bn: "বিক্ষেপকারী বাতাস", verses: 60, makki: true },
    Surah { number: 52, name: "At-Tur", arabic: "الطور", meaning_en: "The Mount", meaning_bn: "পাহাড়", verses: 49, makki: true },
    Surah { number: 53, name: "An-Najm", arabic: "النجم", meaning_en: "The Star", meaning_bn: "তারা", verses: 62, makki: true },
    Surah { number: 54, name: "Al-Qamar", arabic: "القمر", meaning_en: "The Moon", meaning_bn: "চাঁদ", verses: 55, makki: true },
    Surah { number: 55, name: "Ar-Rahman", arabic: "الرحمن", meaning_en: "The Beneficent", meaning_bn: "পরম করুণাময়", verses: 78, makki: false },
    Surah { number: 56, name: "Al-Waqi'ah", arabic: "الواقعة", meaning_en: "The Inevitable", meaning_bn: "নিশ্চিত ঘটনা", verses: 96, makki: true },
    Surah { number: 57, name: "Al-Hadid", arabic: "الحديد", meaning_en: "The Iron", meaning_bn: "লোহা", verses: 29, makki: false },
    Surah { number: 58, name: "Al-Mujadila", arabic: "المجادلة", meaning_en: "The Pleading Woman", meaning_bn: "অনুযোগকারিণী", verses: 22, makki: false },
    Surah { number: 59, name: "Al-Hashr", arabic: "الحشر", meaning_en: "The Exile", meaning_bn: "সমাবেশ", verses: 24, makki: false },
    Surah { number: 60, name: "Al-Mumtahanah", arabic: "الممتحنة", meaning_en: "She that is to be examined", meaning_bn: "নারী, যাকে পরীক্ষা করা হবে", verses: 13, makki: false },
    Surah { number: 61, name: "As-Saf", arabic: "الصف", meaning_en: "The Ranks", meaning_bn: "সারিবদ্ধ সৈন্যদল", verses: 14, makki: false },
    Surah { number: 62, name: "Al-Jumu'ah", arabic: "الجمعة", meaning_en: "The Congregation, Friday", meaning_bn: "সম্মেলন/শুক্রবার", verses: 11, makki: false },
    Surah { number: 63, name: "Al-Munafiqun", arabic: "المنافقون", meaning_en: "The Hypocrites", meaning_bn: "ভণ্ড বিশ্বাসী", verses: 11, makki: false },
    Surah { number: 64, name: "At-Taghabun", arabic: "التغابن", meaning_en: "The Mutual Disillusion", meaning_bn: "মোহ অপসারণ", verses: 18, makki: false },
    Surah { number: 65, name: "At-Talaq", arabic: "الطلاق", meaning_en: "The Divorce", meaning_bn: "তালাক", verses: 12, makki: false },
    Surah { number: 66, name: "At-Tahrim", arabic: "التحريم", meaning_en: "The Prohibition", meaning_bn: "নিষিদ্ধকরণ", verses: 12, makki: false },
    Surah { number: 67, name: "Al-Mulk", arabic: "الملك", meaning_en: "The Sovereignty", meaning_bn: "সার্বভৌম কর্তৃত্ব", verses: 30, makki: true },
    Surah { number: 68, name: "Al-Qalam", arabic: "القلم", meaning_en: "The Pen", meaning_bn: "কলম", verses: 52, makki: true },
    Surah { number: 69, name: "Al-Haqqah", arabic: "الحاقة", meaning_en: "The Reality", meaning_bn: "নিশ্চিত সত্য", verses: 52, makki: true },
    Surah { number: 70, name: "Al-Ma'arij", arabic: "المعارج", meaning_en: "The Ascending Stairways", meaning_bn: "উন্নয়নের সোপান", verses: 44, makki: true },
    Surah { number: 71, name: "Nuh", arabic: "نوح", meaning_en: "Noah", meaning_bn: "নবী নূহ", verses: 28, makki: true },
    Surah { number: 72, name: "Al-Jinn", arabic: "الجن", meaning_en: "The Jinn", meaning_bn: "জিন সম্প্রদায়", verses: 28, makki: true },
    Surah { number: 73, name: "Al-Muzzammil", arabic: "المزمل", meaning_en: "The Enshrouded One", meaning_bn: "বস্ত্রাচ্ছাদনকারী", verses: 20, makki: true },
    Surah { number: 74, name: "Al-Muddaththir", arabic: "المدثر", meaning_en: "The Cloaked One", meaning_bn: "পোশাক পরিহিত", verses: 56, makki: true },
    Surah { number: 75, name: "Al-Qiyamah", arabic: "القيامة", meaning_en: "The Resurrection", meaning_bn: "পুনরুত্থান", verses: 40, makki: true },
    Surah { number: 76, name: "Al-Insan", arabic: "الانسان", meaning_en: "The Man", meaning_bn: "মানবজাতি", verses: 31, makki: false },
    Surah { number: 77, name: "Al-Mursalat", arabic: "المرسلات", meaning_en: "The Emissaries", meaning_bn: "প্রেরিত পুরুষ", verses: 50, makki: true },
    Surah { number: 78, name: "An-Naba", arabic: "النبإ", meaning_en: "The Tidings", meaning_bn: "মহাসংবাদ", verses: 40, makki: true },
    Surah { number: 79, name: "An-Nazi'at", arabic: "النازعات", meaning_en: "Those who drag forth", meaning_bn: "প্রচেষ্টাকারী", verses: 46, makki: true },
    Surah { number: 80, name: "'Abasa", arabic: "عبس", meaning_en: "He Frowned", meaning_bn: "সে ভ্রু কুঁচকালো", verses: 42, makki: true },
    Surah { number: 81, name: "At-Takwir", arabic: "التكوير", meaning_en: "The Overthrowing", meaning_bn: "অন্ধকারাচ্ছন্ন", verses: 29, makki: true },
    Surah { number: 82, name: "Al-Infitar", arabic: "الإنفطار", meaning_en: "The Cleaving", meaning_bn: "বিদীর্ণ করা", verses: 19, makki: true },
    Surah { number: 83, name: "Al-Mutaffifin", arabic: "المطففين", meaning_en: "The Defrauding", meaning_bn: "প্রতারণা করা", verses: 36, makki: true },
    Surah { number: 84, name: "Al-Inshiqaq", arabic: "الإنشقاق", meaning_en: "The Sundering", meaning_bn: "খণ্ড-বিখণ্ডকরণ", verses: 25, makki: true },
    Surah { number: 85, name: "Al-Buruj", arabic: "البروج", meaning_en: "The Mansions of the Stars", meaning_bn: "নক্ষত্রপুঞ্জ", verses: 22, makki: true },
    Surah { number: 86, name: "At-Tariq", arabic: "الطارق", meaning_en: "The Nightcommer", meaning_bn: "রাতের আগন্তুক", verses: 17, makki: true },
    Surah { number: 87, name: "Al-A'la", arabic: "الأعلى", meaning_en: "The Most High", meaning_bn: "সর্বোন্নত", verses: 19, makki: true },
    Surah { number: 88, name: "Al-Ghashiyah", arabic: "الغاشية", meaning_en: "The Overwhelming", meaning_bn: "বিহ্বলকর ঘটনা", verses: 26, makki: true },
    Surah { number: 89, name: "Al-Fajr", arabic: "الفجر", meaning_en: "The Dawn", meaning_bn: "ভোরবেলা", verses: 30, makki: true },
    Surah { number: 90, name: "Al-Balad", arabic: "البلد", meaning_en: "The City", meaning_bn: "নগর", verses: 20, makki: true },
    Surah { number: 91, name: "Ash-Shams", arabic: "الشمس", meaning_en: "The Sun", meaning_bn: "সূর্য", verses: 15, makki: true },
    Surah { number: 92, name: "Al-Layl", arabic: "الليل", meaning_en: "The Night", meaning_bn: "রাত", verses: 21, makki: true },
    Surah { number: 93, name: "Ad-Duhaa", arabic: "الضحى", meaning_en: "The Morning Hours", meaning_bn: "পূর্বাহ্নের সুর্যকিরণ", verses: 11, makki: true },
    Surah { number: 94, name: "Ash-Sharh", arabic: "الشرح", meaning_en: "The Relief", meaning_bn: "বক্ষ প্রশস্তকরণ", verses: 8, makki: true },
    Surah { number: 95, name: "At-Tin", arabic: "التين", meaning_en: "The Fig", meaning_bn: "ডুমুর", verses: 8, makki: true },
    Surah { number: 96, name: "Al-'Alaq", arabic: "العلق", meaning_en: "The Clot", meaning_bn: "রক্তপিণ্ড", verses: 19, makki: true },
    Surah { number: 97, name: "Al-Qadr", arabic: "القدر", meaning_en: "The Power", meaning_bn: "মহিমান্বিত", verses: 5, makki: true },
    Surah { number: 98, name: "Al-Bayyinah", arabic: "البينة", meaning_en: "The Clear Proof", meaning_bn: "প্রমাণ", verses: 8, makki: false },
    Surah { number: 99, name: "Az-Zalzalah", arabic: "الزلزلة", meaning_en: "The Earthquake", meaning_bn: "ভূমিকম্প", verses: 8, makki: false },
    Surah { number: 100, name: "Al-'Adiyat", arabic: "العاديات", meaning_en: "The Courser", meaning_bn: "অভিযানকারী", verses: 11, makki: true },
    Surah { number: 101, name: "Al-Qari'ah", arabic: "القارعة", meaning_en: "The Calamity", meaning_bn: "মহাসংকট", verses: 11, makki: true },
    Surah { number: 102, name: "At-Takathur", arabic: "التكاثر", meaning_en: "The Rivalry in world increase", meaning_bn: "প্রাচুর্যের প্রতিযোগিতা", verses: 8, makki: true },
    Surah { number: 103, name: "Al-'Asr", arabic: "العصر", meaning_en: "The Declining Day", meaning_bn: "সময়", verses: 3, makki: true },
    Surah { number: 104, name: "Al-Humazah", arabic: "الهمزة", meaning_en: "The Traducer", meaning_bn: "পরনিন্দাকারী", verses: 9, makki: true },
    Surah { number: 105, name: "Al-Fil", arabic: "الفيل", meaning_en: "The Elephant", meaning_bn: "হাতি", verses: 5, makki: true },
    Surah { number: 106, name: "Quraysh", arabic: "قريش", meaning_en: "Quraysh", meaning_bn: "কুরাইশ গোত্র", verses: 4, makki: true },
    Surah { number: 107, name: "Al-Ma'un", arabic: "الماعون", meaning_en: "The Small kindnesses", meaning_bn: "সাহায্য সহায়তা", verses: 7, makki: true },
    Surah { number: 108, name: "Al-Kawthar", arabic: "الكوثر", meaning_en: "The Abundance", meaning_bn: "কাউসার/প্রাচুর্য", verses: 3, makki: true },
    Surah { number: 109, name: "Al-Kafirun", arabic: "الكافرون", meaning_en: "The Disbelievers", meaning_bn: "অবিশ্বাসী", verses: 6, makki: true },
    Surah { number: 110, name: "An-Nasr", arabic: "النصر", meaning_en: "The Divine Support", meaning_bn: "সাহায্য", verses: 3, makki: false },
    Surah { number: 111, name: "Al-Masad", arabic: "المسد", meaning_en: "The Palm Fiber", meaning_bn: "খেজুরের পাকানো (রশি)", verses: 5, makki: true },
    Surah { number: 112, name: "Al-Ikhlas", arabic: "الإخلاص", meaning_en: "The Sincerity", meaning_bn: "আন্তরিকতা", verses: 4, makki: true },
    Surah { number: 113, name: "Al-Falaq", arabic: "الفلق", meaning_en: "The Daybreak", meaning_bn: "নিশিভোর", verses: 5, makki: true },
    Surah { number: 114, name: "An-Nas", arabic: "الناس", meaning_en: "Mankind", meaning_bn: "মানুষ জাতি", verses: 6, makki: true },
];

#[cfg(test)]
mod tests {
    #[test]
    fn table_is_complete() {
        assert_eq!(super::SURAHS.len(), 114);
        assert_eq!(super::surah(1).unwrap().verses, 7);
        assert_eq!(super::surah(18).unwrap().name, "Al-Kahf");
        assert_eq!(super::SURAHS.iter().map(|s| s.verses).sum::<u32>(), 6236);
        assert!(super::surah(115).is_none() && super::surah(0).is_none());
    }
}
