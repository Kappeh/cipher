use diesel::prelude::*;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use cipher_core::repository::staff_role_repository::StaffRoleRepository;
use cipher_core::repository::RepositoryError;

use crate::mysql::schema::staff_roles;
use crate::BackendError;

use super::MysqlRepository;

#[async_trait::async_trait]
impl StaffRoleRepository for MysqlRepository<'_> {
    type BackendError = BackendError;

    async fn is_staff_role(&mut self, id: u64) -> Result<bool, RepositoryError<Self::BackendError>> {
        let model_id = id as i64;

        staff_roles::dsl::staff_roles
            .filter(staff_roles::dsl::discord_role_id.eq(model_id))
            .first::<ModelStaffRole>(&mut self.conn)
            .await
            .optional()
            .map(|option| option.is_some())
            .map_err(|err| RepositoryError(BackendError::from(err)))
    }

    async fn staff_roles(&mut self) -> Result<Vec<u64>, RepositoryError<Self::BackendError>> {
        let model_results = staff_roles::dsl::staff_roles
            .select(ModelStaffRole::as_select())
            .load(&mut self.conn)
            .await
            .map_err(|err| RepositoryError(BackendError::from(err)))?;

        let results = model_results.into_iter()
            .map(|r| r.discord_role_id as u64)
            .collect();

        Ok(results)
    }

    async fn staff_roles_contains(&mut self, ids: &[u64]) -> Result<bool, RepositoryError<Self::BackendError>> {
        let model_ids: Vec<_> = ids.into_iter().map(|&id| id as i64).collect();

        staff_roles::dsl::staff_roles
            .filter(staff_roles::dsl::discord_role_id.eq_any(&model_ids))
            .first::<ModelStaffRole>(&mut self.conn)
            .await
            .optional()
            .map(|option| option.is_some())
            .map_err(|err| RepositoryError(BackendError::from(err)))
    }

    async fn set_staff_role(&mut self, id: u64) -> Result<(), RepositoryError<Self::BackendError>> {
        let new_staff_role = ModelNewStaffRole {
            discord_role_id: id as i64,
        };

        self.conn
            .transaction::<_, diesel::result::Error, _>(|conn| async move {
                diesel::insert_into(staff_roles::table)
                    .values(&new_staff_role)
                    .on_conflict_do_nothing()
                    .execute(conn)
                    .await
            }.scope_boxed())
            .await
            .map_err(|err| RepositoryError(BackendError::from(err)))?;

        Ok(())
    }

    async fn unset_staff_role(&mut self, id: u64) -> Result<(), RepositoryError<Self::BackendError>> {
        let model_id = id as i64;

        self.conn
            .transaction::<_, diesel::result::Error, _>(move |conn| async move {
                diesel::delete(staff_roles::dsl::staff_roles.filter(staff_roles::dsl::discord_role_id.eq(model_id)))
                    .execute(conn)
                    .await
            }.scope_boxed())
            .await
            .map_err(|err| RepositoryError(BackendError::from(err)))?;

        Ok(())
    }
}

#[derive(Queryable, Selectable, AsChangeset)]
#[diesel(table_name = staff_roles)]
#[diesel(treat_none_as_null = true)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
struct ModelStaffRole {
    #[allow(unused)]
    id: i32,
    discord_role_id: i64,
}

#[derive(Insertable)]
#[diesel(table_name = staff_roles)]
#[diesel(treat_none_as_null = true)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
struct ModelNewStaffRole {
    discord_role_id: i64,
}
