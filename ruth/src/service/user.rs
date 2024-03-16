use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{Error, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher};
use sea_orm::{
  ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;
use entity::prelude::{Password, User};
use entity::{password, user};

pub(crate) struct UserService {
  db: DatabaseConnection,
  password_verifier: Argon2<'static>,
}

impl UserService {
  pub(crate) fn new(db: DatabaseConnection) -> Self {
    Self {
      db,
      password_verifier: Argon2::default(),
    }
  }

  pub(crate) async fn by_id(&self, id: Uuid) -> anyhow::Result<Option<user::Model>> {
    let user = User::find_by_id(id).one(&self.db).await?;
    Ok(user)
  }

  pub(crate) async fn register(
    &self,
    name: String,
    email: String,
    display_name: String,
    password: &[u8],
  ) -> anyhow::Result<user::Model> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = self.password_verifier.hash_password(password, &salt)?;

    let user = user::ActiveModel {
      id: ActiveValue::NotSet,
      created: ActiveValue::NotSet,
      updated: ActiveValue::NotSet,
      name: ActiveValue::Set(name),
      email: ActiveValue::Set(email),
      display_name: ActiveValue::Set(display_name),
    };

    let user = user.insert(&self.db).await?;

    let password = password::ActiveModel {
      id: ActiveValue::Set(user.id),
      created: ActiveValue::NotSet,
      updated: ActiveValue::NotSet,
      hash: ActiveValue::Set(hash.to_string()),
    };

    password.insert(&self.db).await?;

    Ok(user)
  }

  pub(crate) async fn password_auth(
    &self,
    name: &str,
    password: &[u8],
  ) -> anyhow::Result<Option<user::Model>> {
    let result = Password::find()
      .inner_join(User)
      .filter(user::Column::Name.eq(name).or(user::Column::Email.eq(name)))
      .select_also(User)
      .one(&self.db)
      .await?;

    match result {
      Some((password::Model { hash, .. }, Some(user))) => {
        match PasswordHash::new(&hash)?.verify_password(&[&Argon2::default()], password) {
          Ok(_) => Ok(Some(user)),
          Err(Error::Password) => Ok(None),
          Err(err) => Err(err.into()),
        }
      }
      _ => Ok(None),
    }
  }
}
