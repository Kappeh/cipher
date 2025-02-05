use cipher_core::repository::RepositoryProvider;
use poise::CreateReply;
use serenity::all::Color;
use serenity::all::CreateActionRow;
use serenity::all::CreateButton;
use serenity::all::CreateEmbed;
use serenity::all::CreateEmbedAuthor;
use serenity::all::Member;
use uuid::Uuid;

use crate::app::AppContext;
use crate::app::AppError;

#[poise::command(slash_command, guild_only)]
pub async fn edit<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    option_member: Option<Member>,
) -> Result<(), AppError<R::BackendError>> {
    let member = match option_member {
        Some(member) => {
            if member.user.id != ctx.author().id {
                match crate::checks::is_staff(ctx.into()).await {
                    Ok(true) => {},
                    Ok(false) | Err(AppError::StaffOnly { command_name: _ }) => {

                    }
                    Err(err) => return Err(err)
                }
            }
            member
        }
        None => ctx
            .author_member()
            .await
            .ok_or(AppError::UnknownCacheOrHttpError)?
            .into_owned(),
    };

    let profile = Profile {
        color: member.colour(ctx).unwrap_or(crate::utils::bot_color(&ctx).await),
        author_display_name: member.display_name().to_string(),
        author_icon_url: crate::utils::member_avatar_url(&member),

        thumbnail_url: None,
        image_url: None,

        trainer_class: None,
        nature: None,
        partner_pokemon: None,
        favourite_food: None,
        starting_region: None,
        likes: None,
        quotes: None,

        pokemon_go_code: Some("0000 0000 0000".to_string()),
        pokemon_pocket_code: Some("0000 0000 0000 0000".to_string()),
        switch_code: Some("SW-0000-0000-0000".to_string()),
    };

    let pokemon_info_button_id = Uuid::new_v4().to_string();
    let personal_info_button_id = Uuid::new_v4().to_string();
    let codes_button_id = Uuid::new_v4().to_string();
    let edit_buttons = CreateActionRow::Buttons(vec![
        CreateButton::new(&pokemon_info_button_id).label("Edit Pokémon Info"),
        CreateButton::new(&personal_info_button_id).label("Edit Personal Info"),
        CreateButton::new(&codes_button_id).label("Edit Friend Codes"),
    ]);

    let reply = CreateReply::default()
        .embed(profile.create_embed())
        .components(vec![edit_buttons])
        .ephemeral(true);

    ctx.send(reply).await?;

    Ok(())
}

struct Profile {
    color: Color,
    author_display_name: String,
    author_icon_url: String,

    thumbnail_url: Option<String>,
    image_url: Option<String>,

    trainer_class: Option<String>,
    nature: Option<String>,
    partner_pokemon: Option<String>,
    favourite_food: Option<String>,
    starting_region: Option<String>,
    likes: Option<String>,
    quotes: Option<String>,

    pokemon_go_code: Option<String>,
    pokemon_pocket_code: Option<String>,
    switch_code: Option<String>,
}

impl Profile {
    pub fn create_embed(self) -> CreateEmbed {
        let embed_author = CreateEmbedAuthor::new(self.author_display_name)
            .icon_url(self.author_icon_url);

        let mut embed = CreateEmbed::new()
            .author(embed_author)
            .color(self.color);

        if let Some(thumbnail_url) = self.thumbnail_url {
            embed = embed.thumbnail(thumbnail_url);
        }
        if let Some(image_url) = self.image_url {
            embed = embed.image(image_url)
        }

        let mut is_profile_empty = true;
        if let Some(trainer_class) = self.trainer_class {
            embed = embed.field("Trainer Class", trainer_class, true);
            is_profile_empty = false;
        }
        if let Some(nature) = self.nature {
            embed = embed.field("Nature", nature, true);
            is_profile_empty = false;
        }
        if let Some(partner_pokemon) = self.partner_pokemon {
            embed = embed.field("Partner Pokémon", partner_pokemon, true);
            is_profile_empty = false;
        }
        if let Some(favourite_food) = self.favourite_food {
            embed = embed.field("Favourite Food", favourite_food, true);
            is_profile_empty = false;
        }
        if let Some(starting_region) = self.starting_region {
            embed = embed.field("Starting Region", starting_region, true);
            is_profile_empty = false;
        }
        if let Some(likes) = self.likes {
            embed = embed.field("Likes", likes, true);
            is_profile_empty = false;
        }
        if let Some(quotes) = self.quotes {
            embed = embed.field("Quotes", quotes, false);
            is_profile_empty = false;
        }

        let is_codes_empty
            = self.pokemon_go_code.is_none()
            && self.pokemon_pocket_code.is_none()
            && self.switch_code.is_none();

        match (is_profile_empty, is_codes_empty) {
            (true, true) => embed = embed.description("No information to show."),
            (false, true) => embed = embed.description("**User Profile**"),
            (true, false) => embed = embed.description("**Friend Codes**"),
            (false, false) => {
                embed = embed
                    .description("**User Profile**")
                    .field("\u{200E}", "**Friend Codes**", false); // Invisible character to use title as a spacer
            },
        }

        if let Some(pokemon_go_code) = self.pokemon_go_code {
            embed = embed.field(":PokemonGo: Pokémon Go Friend Code", pokemon_go_code, false);
        }
        if let Some(pokemon_pocket_code) = self.pokemon_pocket_code {
            embed = embed.field(":Pokeball: Pokémon TCG Pocket Friend Code", pokemon_pocket_code, false);
        }
        if let Some(switch_code) = self.switch_code {
            embed = embed.field(":switch: Nintendo Switch Friend Code", switch_code, false);
        }

        embed
    }
}
