#![doc = include_str!("../../README.md")]

pub use derive_wizard_macro::*;
pub use requestty::{Answers, ExpandItem, ListItem, Question, prompt_one};

pub trait Wizard: Sized {
    fn wizard(backend: impl Promptable) -> Self;

    fn wizard_with_message(message: &str, backend: impl Promptable) -> Self {
        let _ = message;
        Self::wizard(backend)
    }

    fn wizard_with_defaults(self, _backend: impl Promptable) -> Self {
        self
    }
}

pub trait Promptable: Clone {
    type Error;
    fn input<V>(
        &self,
        id: String,
        message: String,
        validate_on_submit: V,
        validate_on_key: V,
        prefill: Option<String>,
    ) -> Result<String, Self::Error>
    where
        V: Fn(&str) -> Result<(), String>;

    fn integer_u8<V>(
        &self,
        id: String,
        message: String,
        validate_on_submit: V,
        validate_on_key: V,
        prefill: Option<u8>,
    ) -> Result<u8, Self::Error>
    where
        V: Fn(u8) -> Result<(), String>;

    fn confirm(
        &self,
        id: String,
        message: String,
        prefill: Option<bool>,
    ) -> Result<bool, Self::Error>;

    fn select(
        &self,
        id: String,
        message: String,
        choices: Vec<String>,
        default: Option<usize>,
    ) -> Result<String, Self::Error>;
}

#[derive(Clone)]
pub struct RequesttyWizard;

impl Promptable for RequesttyWizard {
    type Error = requestty::ErrorKind;

    fn input<V>(
        &self,
        id: String,
        message: String,
        validate_on_submit: V,
        validate_on_key: V,
        prefill: Option<String>,
    ) -> Result<String, Self::Error>
    where
        V: Fn(&str) -> Result<(), String>,
    {
        let question = Question::input(id)
            .message(message)
            .validate(|input, _answers| validate_on_submit(input))
            .validate_on_key(|input, _answers| validate_on_key(input).is_ok());
        let question = if let Some(prefill) = prefill {
            question.default(prefill)
        } else {
            question
        };
        let question = question.build();

        let answer = prompt_one(question).unwrap().try_into_string().unwrap();

        Ok(answer)
    }

    fn integer_u8<V>(
        &self,
        id: String,
        message: String,
        validate_on_submit: V,
        validate_on_key: V,
        prefill: Option<u8>,
    ) -> Result<u8, Self::Error>
    where
        V: Fn(u8) -> Result<(), String>,
    {
        let question = Question::int(id)
            .message(message)
            .validate(|input, _answers| validate_on_submit(u8::try_from(input).unwrap()))
            .validate_on_key(|input, _answers| {
                validate_on_key(u8::try_from(input).unwrap()).is_ok()
            });
        let question = if let Some(prefill) = prefill {
            question.default(prefill.into())
        } else {
            question
        };
        let question = question.build();
        let answer = prompt_one(question).unwrap().try_into_int().unwrap() as u8;

        Ok(answer)
    }

    fn confirm(
        &self,
        id: String,
        message: String,
        prefill: Option<bool>,
    ) -> Result<bool, Self::Error> {
        let answer = Question::confirm(id).message(message);
        let answer = if let Some(prefill) = prefill {
            answer.default(prefill)
        } else {
            answer
        };
        let answer = prompt_one(answer).unwrap().try_into_bool().unwrap();

        Ok(answer)
    }

    fn select(
        &self,
        id: String,
        message: String,
        choices: Vec<String>,
        default: Option<usize>,
    ) -> Result<String, Self::Error> {
        let question = Question::select(id).message(message).choices(choices);
        let question = if let Some(default) = default {
            question.default(default)
        } else {
            question
        };
        let answer = prompt_one(question).unwrap().try_into_list_item().unwrap();
        Ok(answer.text.to_string())
    }
}
