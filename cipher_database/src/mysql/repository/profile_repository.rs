use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use cipher_core::repository::profile_repository::NewProfile;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use cipher_core::repository::profile_repository::Profile;
use cipher_core::repository::profile_repository::ProfileRepository;
use cipher_core::repository::RepositoryError;
use diesel::prelude::*;

use crate::mysql::schema::profiles;
use crate::mysql::schema::users;
use crate::BackendError;

use super::MysqlRepository;

#[async_trait::async_trait]
impl ProfileRepository for MysqlRepository<'_> {
    type BackendError = BackendError;

    async fn insert_profile(&mut self, new_profile: NewProfile) -> Result<Profile, RepositoryError<Self::BackendError>> {
        let model_new_profile = ModelNewProfile::from(new_profile);
        self.conn
            .transaction::<_, diesel::result::Error, _>(|conn| async move {
                diesel::update(profiles::table)
                    .filter(profiles::user_id.eq(model_new_profile.user_id))
                    .set(profiles::is_active.eq(false))
                    .execute(conn)
                    .await?;

                diesel::insert_into(profiles::table)
                    .values(&model_new_profile)
                    .execute(conn)
                    .await?;

                profiles::table
                    .filter(profiles::is_active.eq(true))
                    .select(ModelProfile::as_select())
                    .first(conn)
                    .await
            }.scope_boxed())
            .await
            .map(Profile::from)
            .map_err(|err| RepositoryError(BackendError::from(err)))
    }

    async fn profile(&mut self, id: i32) -> Result<Option<Profile>, RepositoryError<Self::BackendError>> {
        profiles::dsl::profiles.find(id)
            .first::<ModelProfile>(&mut self.conn)
            .await
            .map(Profile::from)
            .optional()
            .map_err(|err| RepositoryError(BackendError::from(err)))
    }

    async fn active_profile(&mut self, user_id: i32) -> Result<Option<Profile>, RepositoryError<Self::BackendError>> {
        profiles::dsl::profiles
            .filter(profiles::user_id.eq(user_id))
            .filter(profiles::is_active.eq(true))
            .first::<ModelProfile>(&mut self.conn)
            .await
            .map(Profile::from)
            .optional()
            .map_err(|err| RepositoryError(BackendError::from(err)))
    }

    async fn active_profile_by_discord_id(&mut self, discord_user_id: u64) -> Result<Option<Profile>, RepositoryError<Self::BackendError>> {
        let model_discord_user_id = discord_user_id as i64;
        profiles::table
            .inner_join(users::table)
            .filter(users::discord_user_id.eq(model_discord_user_id))
            .filter(profiles::is_active.eq(true))
            .select(ModelProfile::as_select())
            .first(&mut self.conn)
            .await
            .map(Profile::from)
            .optional()
            .map_err(|err| RepositoryError(BackendError::from(err)))
    }

    async fn profiles_by_user_id(&mut self, user_id: i32) -> Result<Vec<Profile>, RepositoryError<Self::BackendError>> {
        let results: Vec<_> = profiles::dsl::profiles
            .filter(profiles::user_id.eq(user_id))
            .order(profiles::created_at.desc())
            .get_results::<ModelProfile>(&mut self.conn)
            .await
            .map_err(|err| RepositoryError(BackendError::from(err)))?
            .into_iter()
            .map(Profile::from)
            .collect();

        Ok(results)
    }

    async fn profiles_by_discord_id(&mut self, discord_user_id: u64) -> Result<Vec<Profile>, RepositoryError<Self::BackendError>> {
        let model_discord_user_id = discord_user_id as i64;
        let results = profiles::table
            .inner_join(users::table)
            .filter(users::discord_user_id.eq(model_discord_user_id))
            .order(profiles::created_at.desc())
            .select(ModelProfile::as_select())
            .get_results(&mut self.conn)
            .await
            .map_err(|err| RepositoryError(BackendError::from(err)))?
            .into_iter()
            .map(Profile::from)
            .collect();

        Ok(results)
    }

    async fn set_active_profile(&mut self, user_id: i32, profile_id: i32) -> Result<bool, RepositoryError<Self::BackendError>> {
        self.conn
            .transaction::<_, diesel::result::Error, _>(move |conn| async move {
                let num_affected = diesel::update(profiles::table.find(profile_id))
                    .set(profiles::is_active.eq(true))
                    .execute(conn)
                    .await?;

                if num_affected == 0 {
                    return Ok(false);
                }

                diesel::update(profiles::table)
                    .filter(profiles::user_id.eq(user_id))
                    .filter(profiles::id.ne(profile_id))
                    .filter(profiles::is_active.eq(true))
                    .set(profiles::is_active.eq(false))
                    .execute(conn)
                    .await?;

                Ok(true)
            }.scope_boxed())
            .await
            .map_err(|err| RepositoryError(BackendError::from(err)))
    }
}

#[derive(Queryable, Selectable, AsChangeset)]
#[diesel(table_name = profiles)]
#[diesel(belongs_to(ModelUser))]
#[diesel(treat_none_as_null = true)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelProfile {
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

    pub created_at: NaiveDateTime,
    pub is_active: bool,
}

impl From<ModelProfile> for Profile {
    fn from(value: ModelProfile) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,

            thumbnail_url: value.thumbnail_url,
            image_url: value.image_url,

            trainer_class: value.trainer_class,
            nature: value.nature,
            partner_pokemon: value.partner_pokemon,
            starting_region: value.starting_region,
            favourite_food: value.favourite_food,
            likes: value.likes,
            quotes: value.quotes,

            pokemon_go_code: value.pokemon_go_code,
            pokemon_pocket_code: value.pokemon_pocket_code,
            switch_code: value.switch_code,

            created_at: DateTime::from_naive_utc_and_offset(value.created_at, Utc),
            is_active: value.is_active,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = profiles)]
#[diesel(treat_none_as_null = true)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelNewProfile {
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

    pub created_at: NaiveDateTime,
    pub is_active: bool,
}

impl From<NewProfile> for ModelNewProfile {
    fn from(value: NewProfile) -> Self {
        Self {
            user_id: value.user_id,

            thumbnail_url: value.thumbnail_url,
            image_url: value.image_url,

            trainer_class: value.trainer_class,
            nature: value.nature,
            partner_pokemon: value.partner_pokemon,
            starting_region: value.starting_region,
            favourite_food: value.favourite_food,
            likes: value.likes,
            quotes: value.quotes,

            pokemon_go_code: value.pokemon_go_code,
            pokemon_pocket_code: value.pokemon_pocket_code,
            switch_code: value.switch_code,

            created_at: Utc::now().naive_utc(),
            is_active: true,
        }
    }
}
