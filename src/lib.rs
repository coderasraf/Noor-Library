//! Noor Library: an Islamic media library. Lectures, series and Quran
//! recitations from server folders, local folders, YouTube and podcast
//! feeds, grouped, searchable and playable in the app or an external player.
//!
//! Two front ends share this library: the `noor` CLI (src/main.rs) and
//! the `noor-library` app (src/desktop.rs).

pub mod assistant;
pub mod config;
pub mod crawler;
pub mod feed;
pub mod index;
pub mod jobs;
pub mod library;
pub mod naming;
pub mod player;
pub mod quran;
pub mod topics;
pub mod web;
pub mod youtube;
