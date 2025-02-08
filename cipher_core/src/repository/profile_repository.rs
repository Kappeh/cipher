use chrono::DateTime;
use chrono::Utc;

use super::RepositoryError;

/// A repository trait for managing user profiles, supporting asynchronous operations.
///
/// This trait defines the required operations for interacting with user profiles,
/// including creation, retrieval, history management, and version rollback. It is
/// designed to be implemented for various backends, such as databases, using Diesel or Diesel Async.
///
/// # Type Parameters
/// - `BackendError`: Represents the error type returned by the underlying backend implementation.
///
/// # Errors
/// All methods return a `RepositoryError<Self::BackendError>`, which encapsulates
/// errors specific to the backend implementation.
#[async_trait::async_trait]
pub trait ProfileRepository {
    /// The associated error type returned by backend operations.
    type BackendError: std::error::Error;

    /// Inserts a new profile into the repository and marks it as the active profile for the user it belongs to.
    ///
    /// # Arguments
    /// * `new_profile` - The profile data to insert.
    ///
    /// # Returns
    /// * `Ok(Profile)` - The inserted profile with its assigned ID.
    /// * `Err(RepositoryError<Self::BackendError>)` - If the operation fails.
    async fn insert_profile(&mut self, new_profile: NewProfile) -> Result<Profile, RepositoryError<Self::BackendError>>;

    /// Retrieves a profile by its unique ID.
    ///
    /// # Arguments
    /// * `id` - The unique profile ID.
    ///
    /// # Returns
    /// * `Ok(Some(Profile))` - If a profile with the given ID exists.
    /// * `Ok(None)` - If no profile is found.
    /// * `Err(RepositoryError<Self::BackendError>)` - If an error occurs.
    async fn profile(&mut self, id: i32) -> Result<Option<Profile>, RepositoryError<Self::BackendError>>;

    /// Retrieves the active profile associated with a given user ID.
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user whose profile is being retrieved.
    ///
    /// # Returns
    /// * `Ok(Some(Profile))` - The users active profile for the user if it exists.
    /// * `Ok(None)` - If no active profile is found for the user.
    /// * `Err(RepositoryError<Self::BackendError>)` - If an error occurs.
    async fn active_profile(&mut self, user_id: i32) -> Result<Option<Profile>, RepositoryError<Self::BackendError>>;

    /// Retrieves the active profile associated with a given Discord user ID.
    ///
    /// # Arguments
    /// * `discord_user_id` - The Discord user's unique identifier.
    ///
    /// # Returns
    /// * `Ok(Some(Profile))` - The users active profile for the user if it exists.
    /// * `Ok(None)` - If no active profile is found for the user.
    /// * `Err(RepositoryError<Self::BackendError>)` - If an error occurs.
    async fn active_profile_by_discord_id(&mut self, discord_user_id: u64) -> Result<Option<Profile>, RepositoryError<Self::BackendError>>;

    /// Retrieves the full profile history for a given user.
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user whose profile history is being retrieved.
    ///
    /// # Returns
    /// * `Ok(Vec<Profile>)` - A list of all past versions of the user's profile.
    /// * `Err(RepositoryError<Self::BackendError>)` - If an error occurs.
    async fn profiles_by_user_id(&mut self, user_id: i32) -> Result<Vec<Profile>, RepositoryError<Self::BackendError>>;

    /// Retrieves the full profile history for a given Discord user.
    ///
    /// # Arguments
    /// * `discord_user_id` - The Discord user's unique identifier.
    ///
    /// # Returns
    /// * `Ok(Vec<Profile>)` - A list of all past versions of the user's profile.
    /// * `Err(RepositoryError<Self::BackendError>)` - If an error occurs.
    async fn profiles_by_discord_id(&mut self, discord_user_id: u64) -> Result<Vec<Profile>, RepositoryError<Self::BackendError>>;

    /// Sets an profile version as active.
    ///
    /// This function marks `profile_id` as the active profile
    /// and deactivates all other profiles for the user.
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user.
    /// * `profile_id` - The profile ID to mark as active.
    ///
    /// # Returns
    /// * `Ok(false)` - If the specified profile does not exist.
    /// * `Ok(true)` - If the operation was successful.
    /// * `Err(RepositoryError<Self::BackendError>)` - If the operation fails.
    async fn set_active_profile(&mut self, user_id: i32, profile_id: i32) -> Result<bool, RepositoryError<Self::BackendError>>;
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub id: i32,
    pub user_id: i32,

    pub thumbnail_url: Option<String>,
    pub image_url: Option<String>,

    pub trainer_class: Option<String>,
    pub nature: Option<String>,
    pub partner_pokemon: Option<String>,
    pub starting_region: Option<String>,
    pub favourite_food: Option<String>,
    pub likes: Option<String>,
    pub quotes: Option<String>,

    pub pokemon_go_code: Option<String>,
    pub pokemon_pocket_code: Option<String>,
    pub switch_code: Option<String>,

    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NewProfile {
    pub user_id: i32,

    pub thumbnail_url: Option<String>,
    pub image_url: Option<String>,

    pub trainer_class: Option<String>,
    pub nature: Option<String>,
    pub partner_pokemon: Option<String>,
    pub starting_region: Option<String>,
    pub favourite_food: Option<String>,
    pub likes: Option<String>,
    pub quotes: Option<String>,

    pub pokemon_go_code: Option<String>,
    pub pokemon_pocket_code: Option<String>,
    pub switch_code: Option<String>,
}

impl Profile {
    pub fn into_new(self) -> NewProfile {
        NewProfile {
            user_id: self.user_id,

            thumbnail_url: self.thumbnail_url,
            image_url: self.image_url,

            trainer_class: self.trainer_class,
            nature: self.nature,
            partner_pokemon: self.partner_pokemon,
            starting_region: self.starting_region,
            favourite_food: self.favourite_food,
            likes: self.likes,
            quotes: self.quotes,

            pokemon_go_code: self.pokemon_go_code,
            pokemon_pocket_code: self.pokemon_pocket_code,
            switch_code: self.switch_code,
        }
    }
}
