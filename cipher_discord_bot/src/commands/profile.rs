use std::time::Duration;

use cipher_core::repository::profile_repository::NewProfile;
use cipher_core::repository::profile_repository::Profile;
use cipher_core::repository::profile_repository::ProfileRepository;
use cipher_core::repository::user_repository::NewUser;
use cipher_core::repository::user_repository::UserRepository;
use cipher_core::repository::RepositoryProvider;
use poise::CreateReply;
use poise::ReplyHandle;
use serenity::all::ButtonStyle;
use serenity::all::Color;
use serenity::all::ComponentInteractionCollector;
use serenity::all::CreateActionRow;
use serenity::all::CreateButton;
use serenity::all::CreateEmbed;
use serenity::all::CreateEmbedAuthor;
use serenity::all::CreateInteractionResponse;
use serenity::all::Member;
use serenity::all::User;
use uuid::Uuid;

use crate::app::AppContext;
use crate::app::AppError;

/// Edit and show profiles.
#[poise::command(
    slash_command,
    subcommands(
        "edit",
        "overwrite",
        "show",
    ),
)]
pub async fn profile<R: RepositoryProvider + Send + Sync>(
    _ctx: AppContext<'_, R, R::BackendError>,
) -> Result<(), AppError<R::BackendError>> {
    Ok(())
}

#[poise::command(
    context_menu_command = "Show User Profile",
    guild_only,
)]
pub async fn cmu_profile_show<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    user: User,
) -> Result<(), AppError<R::BackendError>> {
    let guild = match ctx.guild_id() {
        Some(guild) => guild,
        None => return Ok(()),
    };

    let member = guild.member(ctx, user.id).await?;

    show_inner(ctx, member, true).await
}

/// Show your profile or someone else's.
#[poise::command(
    slash_command,
    guild_only,
)]
async fn show<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    #[rename = "member"]
    #[description = "The profile to show."]
    option_member: Option<serenity::all::Member>,
    #[description = "Hide reply from other users. Defaults to True."]
    ephemeral: Option<bool>,
) -> Result<(), AppError<R::BackendError>> {
    let member = match option_member {
        Some(member) => member,
        None => ctx.author_member().await.ok_or(AppError::UnknownCacheOrHttpError)?.into_owned(),
    };

    show_inner(ctx, member, ephemeral.unwrap_or(true)).await
}

#[poise::command(
    slash_command,
    guild_only,
)]
async fn edit<R: RepositoryProvider + Send + Sync>(ctx: AppContext<'_, R, R::BackendError>) -> Result<(), AppError<R::BackendError>> {
    let member = match ctx.author_member().await {
        Some(member) => member,
        None => {
            let embed = CreateEmbed::new()
                .title("Guild Only Command")
                .description("This command can only be used in guilds.")
                .color(crate::utils::bot_color(&ctx).await);

            let reply = CreateReply::default()
                .embed(embed)
                .ephemeral(true);

            ctx.send(reply).await?;

            return Ok(());
        },
    };

    edit_inner(ctx, member.into_owned()).await?;

    Ok(())
}


#[poise::command(
    slash_command,
    guild_only,
    hide_in_help,
    check = "crate::checks::is_staff",
)]
async fn overwrite<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    member: Member,
) -> Result<(), AppError<R::BackendError>> {
    edit_inner(ctx, member).await?;

    Ok(())
}

async fn show_inner<R>(ctx: AppContext<'_, R, R::BackendError>, member: Member, ephemeral: bool) -> Result<(), AppError<R::BackendError>>
where
    R: RepositoryProvider + Send + Sync,
{
    let mut repo = ctx.data.repository().await?;

    let option_profile = repo.active_profile_by_discord_id(member.user.id.get()).await?;
    let embed = ProfileEmbed::from_profile(&ctx, &member, option_profile.as_ref()).await.into_embed();

    let reply = CreateReply::default()
        .embed(embed)
        .ephemeral(ephemeral);

    ctx.send(reply).await?;

    Ok(())
}

async fn edit_inner<R>(ctx: AppContext<'_, R, R::BackendError>, member: Member) -> Result<(), AppError<R::BackendError>>
where
    R: RepositoryProvider + Send + Sync,
{
    let mut repo = ctx.data.repository().await?;

    let mut option_reply_handle: Option<ReplyHandle> = None;
    let mut option_profile = repo.active_profile_by_discord_id(member.user.id.get()).await?.map(Profile::into_new);

    'update_reply: loop {
        let embed = ProfileEmbed::from_new_profile(&ctx, &member, option_profile.as_ref()).await.into_embed();

        let pokemon_info_button_id = Uuid::new_v4().to_string();
        let personal_info_button_id = Uuid::new_v4().to_string();
        let friend_codes_button_id = Uuid::new_v4().to_string();
        let images_button_id = Uuid::new_v4().to_string();
        let save_button_id = Uuid::new_v4().to_string();
        let buttons = CreateActionRow::Buttons(vec![
            CreateButton::new(&pokemon_info_button_id).label("Edit Pokémon Info").style(ButtonStyle::Secondary),
            CreateButton::new(&personal_info_button_id).label("Edit Personal Info").style(ButtonStyle::Secondary),
            CreateButton::new(&friend_codes_button_id).label("Edit Friend Codes").style(ButtonStyle::Secondary),
            CreateButton::new(&images_button_id).label("Edit Images").style(ButtonStyle::Secondary),
            CreateButton::new(&save_button_id).label("Save").style(ButtonStyle::Primary),
        ]);

        let reply = CreateReply::default()
            .embed(embed)
            .components(vec![buttons])
            .ephemeral(true);

        let reply_handle = match option_reply_handle {
            Some(reply_handle) => {
                reply_handle.edit(ctx.into(), reply).await?;
                reply_handle
            },
            None => ctx.send(reply).await?,
        };

        'interaction_response: loop {
            let collector = ComponentInteractionCollector::new(ctx)
                .author_id(ctx.author().id)
                .channel_id(ctx.channel_id())
                .timeout(Duration::from_secs(60));

            let mci = match collector.await {
                Some(mci) => mci,
                None => {
                    let embed = CreateEmbed::new()
                        .title("Editor Timed Out")
                        .description("Your changes have not been saved. Please use `/profile edit` again to continue.")
                        .color(crate::utils::bot_color(&ctx).await);

                    let reply = CreateReply::default()
                        .embed(embed)
                        .components(vec![])
                        .ephemeral(true);

                    reply_handle.edit(ctx.into(), reply).await?;

                    break 'update_reply;
                },
            };

            if mci.data.custom_id == pokemon_info_button_id {
                let option_defaults = option_profile.clone().map(|profile| EditPokemonInfoModal {
                    trainer_class: profile.trainer_class,
                    nature: profile.nature,
                    partner_pokemon: profile.partner_pokemon,
                    starting_region: profile.starting_region,
                });

                let data = match poise::execute_modal_on_component_interaction(ctx, mci.clone(), option_defaults, None).await? {
                    Some(data) => data,
                    None => continue 'interaction_response,
                };

                let profile = match option_profile {
                    Some(mut profile) => {
                        profile.trainer_class = data.trainer_class;
                        profile.nature = data.nature;
                        profile.partner_pokemon = data.partner_pokemon;
                        profile.starting_region = data.starting_region;
                        profile
                    },
                    None => NewProfile {
                        trainer_class: data.trainer_class,
                        nature: data.nature,
                        partner_pokemon: data.partner_pokemon,
                        starting_region: data.starting_region,
                        ..Default::default()
                    },
                };

                option_profile = Some(profile);
                break 'interaction_response;
            }

            if mci.data.custom_id == personal_info_button_id {
                let option_defaults = option_profile.clone().map(|profile| EditPersonalInfoModal {
                    favourite_food: profile.favourite_food,
                    likes: profile.likes,
                    quotes: profile.quotes,
                });

                let data = match poise::execute_modal_on_component_interaction(ctx, mci.clone(), option_defaults, None).await? {
                    Some(data) => data,
                    None => continue 'interaction_response,
                };

                let profile = match option_profile {
                    Some(mut profile) => {
                        profile.favourite_food = data.favourite_food;
                        profile.likes = data.likes;
                        profile.quotes = data.quotes;
                        profile
                    },
                    None => NewProfile {
                        favourite_food: data.favourite_food,
                        likes: data.likes,
                        quotes: data.quotes,
                        ..Default::default()
                    },
                };

                option_profile = Some(profile);
                break 'interaction_response;
            }

            if mci.data.custom_id == friend_codes_button_id {
                let option_defaults = option_profile.clone().map(|profile| EditCodesModal {
                    pokemon_go_code: profile.pokemon_go_code,
                    pokemon_pocket_code: profile.pokemon_pocket_code,
                    switch_code: profile.switch_code,
                });

                let mut data = match poise::execute_modal_on_component_interaction(ctx, mci.clone(), option_defaults, None).await? {
                    Some(data) => data,
                    None => continue 'interaction_response,
                };

                if let Err(errors) = data.validate() {
                    let mut embed_description = String::new();

                    for error in errors {
                        embed_description.push_str(&error);
                        embed_description.push('\n');
                    }
                    embed_description.pop();

                    let embed = CreateEmbed::new()
                        .title("Validation Error")
                        .description(embed_description)
                        .color(Color::RED);

                    let reply = CreateReply::default()
                        .embed(embed)
                        .ephemeral(true);

                    ctx.send(reply).await?;

                    continue 'interaction_response;
                }

                let profile = match option_profile {
                    Some(mut profile) => {
                        profile.pokemon_go_code = data.pokemon_go_code;
                        profile.pokemon_pocket_code = data.pokemon_pocket_code;
                        profile.switch_code = data.switch_code;
                        profile
                    },
                    None => NewProfile {
                        pokemon_go_code: data.pokemon_go_code,
                        pokemon_pocket_code: data.pokemon_pocket_code,
                        switch_code: data.switch_code,
                        ..Default::default()
                    },
                };

                option_profile = Some(profile);
                break 'interaction_response;
            }

            if mci.data.custom_id == images_button_id {
                let option_defaults = option_profile.clone().map(|profile| EditImagesModal {
                    thumbnail_url: profile.thumbnail_url,
                    image_url: profile.image_url,
                });

                let data = match poise::execute_modal_on_component_interaction(ctx, mci.clone(), option_defaults, None).await? {
                    Some(data) => data,
                    None => continue 'interaction_response,
                };

                let profile = match option_profile {
                    Some(mut profile) => {
                        profile.thumbnail_url = data.thumbnail_url;
                        profile.image_url = data.image_url;
                        profile
                    },
                    None => NewProfile {
                        thumbnail_url: data.thumbnail_url,
                        image_url: data.image_url,
                        ..Default::default()
                    },
                };

                option_profile = Some(profile);
                break 'interaction_response;
            }

            if mci.data.custom_id == save_button_id {
                let mut new_profile = match option_profile {
                    Some(new_profile) => new_profile,
                    None => break,
                };

                let discord_user_id = member.user.id.get();
                let user = match repo.user_by_discord_user_id(discord_user_id).await? {
                    Some(user) => user,
                    None => repo.insert_user(NewUser { discord_user_id }).await?,
                };

                new_profile.user_id = user.id;

                repo.insert_profile(new_profile).await?;

                let embed = CreateEmbed::new()
                    .title("Saved")
                    .description("Your changes have been saved successfully!")
                    .color(crate::utils::bot_color(&ctx).await);

                let reply = CreateReply::default()
                    .embed(embed)
                    .components(vec![])
                    .ephemeral(true);

                reply_handle.edit(ctx.into(), reply).await?;

                break 'update_reply;
            }

            mci.create_response(ctx, CreateInteractionResponse::Acknowledge).await?;
        }

        option_reply_handle = Some(reply_handle);
    }

    Ok(())
}

#[derive(Default)]
struct ProfileEmbed {
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

impl ProfileEmbed {
    async fn from_profile<R>(
        ctx: &AppContext<'_, R, R::BackendError>,
        member: &Member,
        option_profile: Option<&Profile>,
    ) -> ProfileEmbed
    where
        R: RepositoryProvider + Send + Sync,
    {
        let avatar_url = crate::utils::member_avatar_url(member);

        let embed_color = match member.colour(ctx) {
            Some(color) => color,
            None => crate::utils::bot_color(ctx).await,
        };

        match option_profile.cloned() {
            Some(profile) => ProfileEmbed {
                color: embed_color,
                author_display_name: member.display_name().to_string(),
                author_icon_url: avatar_url,

                thumbnail_url: profile.thumbnail_url,
                image_url: profile.image_url,

                trainer_class: profile.trainer_class,
                nature: profile.nature,
                partner_pokemon: profile.partner_pokemon,
                favourite_food: profile.favourite_food,
                starting_region: profile.starting_region,
                likes: profile.likes,
                quotes: profile.quotes,

                pokemon_go_code: profile.pokemon_go_code,
                pokemon_pocket_code: profile.pokemon_pocket_code,
                switch_code: profile.switch_code,
            },
            None => ProfileEmbed {
                color: embed_color,
                author_display_name: member.display_name().to_string(),
                author_icon_url: avatar_url,
                ..Default::default()
            },
        }
    }

    async fn from_new_profile<R>(
        ctx: &AppContext<'_, R, R::BackendError>,
        member: &Member,
        option_profile: Option<&NewProfile>,
    ) -> ProfileEmbed
    where
        R: RepositoryProvider + Send + Sync,
    {
        let avatar_url = crate::utils::member_avatar_url(member);

        let embed_color = match member.colour(ctx) {
            Some(color) => color,
            None => crate::utils::bot_color(ctx).await,
        };

        match option_profile.cloned() {
            Some(profile) => ProfileEmbed {
                color: embed_color,
                author_display_name: member.display_name().to_string(),
                author_icon_url: avatar_url,

                thumbnail_url: profile.thumbnail_url,
                image_url: profile.image_url,

                trainer_class: profile.trainer_class,
                nature: profile.nature,
                partner_pokemon: profile.partner_pokemon,
                favourite_food: profile.favourite_food,
                starting_region: profile.starting_region,
                likes: profile.likes,
                quotes: profile.quotes,

                pokemon_go_code: profile.pokemon_go_code,
                pokemon_pocket_code: profile.pokemon_pocket_code,
                switch_code: profile.switch_code,
            },
            None => ProfileEmbed {
                color: embed_color,
                author_display_name: member.display_name().to_string(),
                author_icon_url: avatar_url,
                ..Default::default()
            },
        }
    }

    pub fn into_embed(self) -> CreateEmbed {
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
            embed = embed.field("Pokémon", partner_pokemon, true);
            is_profile_empty = false;
        }
        if let Some(favourite_food) = self.favourite_food {
            embed = embed.field("Favourite Food", favourite_food, true);
            is_profile_empty = false;
        }
        if let Some(starting_region) = self.starting_region {
            embed = embed.field("Region", starting_region, true);
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
            embed = embed.field("<:PokemonGo:961206166812250156> Pokémon Go Friend Code", pokemon_go_code, false);
        }
        if let Some(pokemon_pocket_code) = self.pokemon_pocket_code {
            embed = embed.field("<:Pokeball:961206135535337513> Pokémon TCG Pocket Friend Code", pokemon_pocket_code, false);
        }
        if let Some(switch_code) = self.switch_code {
            embed = embed.field("<:switch:1335457825161220111> Nintendo Switch Friend Code", switch_code, false);
        }

        embed
    }
}

#[derive(Debug, Clone, Default, poise::Modal)]
#[name = "Edit Pokémon Information"]
struct EditPokemonInfoModal {
    #[name = "Trainer Class"]
    trainer_class: Option<String>,
    #[name = "Nature"]
    nature: Option<String>,
    #[name = "Pokémon"]
    partner_pokemon: Option<String>,
    #[name = "Region"]
    starting_region: Option<String>,
}

#[derive(Debug, Clone, Default, poise::Modal)]
#[name = "Edit Personal Information"]
struct EditPersonalInfoModal {
    #[name = "Favourite Food"]
    favourite_food: Option<String>,
    #[name = "Likes"]
    likes: Option<String>,
    #[name = "Quotes"]
    #[paragraph]
    quotes: Option<String>,
}

#[derive(Debug, Clone, Default, poise::Modal)]
#[name = "Edit Friend Codes"]
struct EditCodesModal {
    #[name = "Pokémon Go Friend Code"]
    #[placeholder = "0000 0000 0000"]
    pokemon_go_code: Option<String>,
    #[name = "Pokémon TCG Pocket Friend Code"]
    #[placeholder = "0000 0000 0000 0000"]
    pokemon_pocket_code: Option<String>,
    #[name = "Nintendo Switch Friend Code"]
    #[placeholder = "SW-0000-0000-0000"]
    switch_code: Option<String>,
}

#[derive(Debug, Clone, Default, poise::Modal)]
#[name = "Edit Images"]
struct EditImagesModal {
    #[name = "Thumbnail Image URL"]
    thumbnail_url: Option<String>,
    #[name = "Footer Image URL"]
    image_url: Option<String>,
}

fn parse_pokemon_go_code(code: &str) -> Option<String> {
    let mut chars = code.chars().peekable();

    let mut parsed = String::new();

    for i in 0..3 {
        if i > 0 {
            let c = *chars.peek()?;
            if c == '-' || c == ' ' {
                chars.next();
            }
            parsed.push(' ');
        }

        for _ in 0..4 {
            let c = chars.next()?;
            if !c.is_numeric() {
                return None;
            }
            parsed.push(c);
        }
    }

    chars.next().is_none().then_some(parsed)
}

fn parse_pokemon_pocket_code(code: &str) -> Option<String> {
    let mut chars = code.chars().peekable();

    let mut parsed = String::new();

    for i in 0..4 {
        if i > 0 {
            let c = *chars.peek()?;
            if c == '-' || c == ' ' {
                chars.next();
            }
            parsed.push(' ');
        }

        for _ in 0..4 {
            let c = chars.next()?;
            if !c.is_numeric() {
                return None;
            }
            parsed.push(c);
        }
    }

    chars.next().is_none().then_some(parsed)
}

fn parse_switch_code(code: &str) -> Option<String> {
    let mut chars = code.chars().map(|c| c.to_ascii_uppercase()).peekable();

    let mut parsed = String::from("SW");

    if *chars.peek()? == 'S' {
        chars.next();
        if chars.next()? != 'W' {
            return None;
        }
    }

    for _ in 0..3 {
        let c = *chars.peek()?;
        if c == '-' || c == ' ' {
            chars.next();
        }
        parsed.push('-');

        for _ in 0..4 {
            let c = chars.next()?;
            if !c.is_numeric() {
                return None;
            }
            parsed.push(c);
        }
    }

    chars.next().is_none().then_some(parsed)
}

impl EditCodesModal {
    fn validate(&mut self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Some(code) = &self.pokemon_go_code {
            match parse_pokemon_go_code(code) {
                Some(parsed) => self.pokemon_go_code = Some(parsed),
                None => errors.push(format!("`{}` is not a valid Pokémon Go friend code.", code)),
            };
        }

        if let Some(code) = &self.pokemon_pocket_code {
            match parse_pokemon_pocket_code(code) {
                Some(parsed) => self.pokemon_pocket_code = Some(parsed),
                None => errors.push(format!("`{}` is not a valid Pokémon TCG Pocket friend code.", code)),
            };
        }

        if let Some(code) = &self.switch_code {
            match parse_switch_code(code) {
                Some(parsed) => self.switch_code = Some(parsed),
                None => errors.push(format!("`{}` is not a valid switch friend code.", code)),
            };
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
