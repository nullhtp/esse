//! The one place the method is written down.
//!
//! `cmd-h` answers "what can I press here"; `cmd-shift-h` answers "what do I
//! actually do here" — and answers it as an instruction, in order, from the
//! first move to the one that ends the place. The mechanics already enforce
//! the method; nothing said it out loud (writing-guidance spec, design.md D1).
//!
//! Same contract as the shortcut sheet: summoned only, any key dismisses it
//! without acting, and nothing behind it changes. The steps are fixed prose out
//! of CONCEPT.md — they never read the essay (design.md, D2, D3).

use gpui::{
    actions, div, prelude::*, px, rgb, rgba, App, FocusHandle, KeyDownEvent, SharedString, Window,
};

use crate::keymap::{self, Place};
use crate::theme;

actions!(guidance, [Toggle]);

/// What to say about a place: what it is for, then what to do in it, in order.
struct Method {
    /// The place in one line — the point of standing here at all.
    lead: &'static str,
    /// The steps, first move first, ending with what takes the writer out of
    /// this place. Each carries its own reason: a step nobody believes is a
    /// step nobody follows.
    steps: &'static [&'static str],
}

/// The sheet for the place the writer is standing in. `dismiss` is called on
/// any keypress: the guidance context binds nothing, so a key that would have
/// switched modes or typed a letter only closes it (design.md, D2).
pub fn overlay(
    place: Place,
    focus: &FocusHandle,
    dismiss: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let method = method(place);

    div()
        .key_context(keymap::GUIDANCE)
        .track_focus(focus)
        .occlude()
        .on_key_down(dismiss)
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(VEIL))
        .child(
            div()
                .w(px(CARD_WIDTH))
                .max_w_full()
                .p(px(26.))
                .rounded(px(10.))
                .bg(rgb(theme::BACKGROUND))
                .border_1()
                .border_color(rgb(theme::RULE))
                .text_color(rgb(theme::INK))
                .text_size(px(theme::BODY_SIZE))
                .line_height(px(theme::BODY_SIZE * theme::LINE_SPACING))
                .child(
                    div()
                        .w_full()
                        .pb(px(6.))
                        .text_size(px(theme::SMALL_SIZE))
                        .text_color(rgb(theme::MUTED))
                        .child(place.title()),
                )
                .child(div().w_full().pb(px(12.)).child(method.lead))
                .children(steps(method.steps)),
        )
}

/// The steps, numbered. Each is one paragraph of the card's own width: a
/// definite width is what makes the text wrap at all — laid out as a flex row
/// with the number in a gutter, the paragraph's minimum size is its whole
/// unwrapped length, and the sheet runs off the screen instead.
fn steps(steps: &'static [&'static str]) -> Vec<impl IntoElement> {
    steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            div()
                .w_full()
                .py(px(5.))
                .child(SharedString::from(format!("{}. {step}", index + 1)))
        })
        .collect()
}

/// What to say in each place. An exhaustive match, so a new [`Place`] cannot be
/// added without deciding what the method is there (design.md, D3).
fn method(place: Place) -> &'static Method {
    match place {
        Place::Today => &TODAY,
        Place::ChoosingSpark => &CHOOSING_SPARK,
        Place::Write => &WRITE,
        Place::Edit => &EDIT,
        Place::Finishing => &FINISHING,
        Place::Shelf => &SHELF,
    }
}

/// The daily loop: catch sparks between sessions, spend one session writing
/// (CONCEPT.md, механики 1 и 4).
static TODAY: Method = Method {
    lead: "Здесь всё начинается заново. Одна искра, один заход — из этого и вырастает привычка писать.",
    steps: &[
        "Мелькнула мысль — не дайте ей уйти: строчка в поле искр, и она ваша. Это зерно, а не обязательство.",
        "Готовы — жмите «Писать». Приложение само вспомнит, где вы остановились, или предложит выбрать искру.",
        "Дальше всего двадцать минут. Не «написать эссе», а побыть с текстом — этого достаточно.",
        "Время вышло — выходите со спокойной душой. Заход состоялся, даже если до конца ещё далеко.",
        "День отметится точкой внизу. Пропустили — не страшно: важна не цепочка, а то, что вы возвращаетесь.",
    ],
};

/// Choosing is the cheap part; deliberating over it is the expensive one
/// (CONCEPT.md, механика 2).
static CHOOSING_SPARK: Method = Method {
    lead: "Все ваши мысли уже здесь — осталось выбрать одну. Выбор запускает работу, так пусть он будет лёгким.",
    steps: &[
        "Пробегите список сверху вниз и берите ту, на которой споткнулся взгляд. Она и есть живая.",
        "Не ищите «самую важную». Важной искра становится, пока вы о ней пишете, а не пока её выбираете.",
        "Не бойтесь, что не знаете тему целиком. Никто не знает — текст затем и пишется, чтобы понять.",
        "Выбранная искра станет эссе, остальные дождутся очереди. Копилка не пустеет от одного выбора.",
        "Передумали — выйдите, ничего не начав. Счётчика неудачных заходов здесь нет и не будет.",
    ],
};

/// The mode that exists to make bad text (CONCEPT.md, механика 3).
static WRITE: Method = Method {
    lead: "Здесь можно писать плохо — и нужно. Задача не сделать хорошо, а сделать целиком; хорошо будет потом.",
    steps: &[
        "Начните с любой первой фразы. Она не обязана быть удачной — её всё равно почти наверняка перепишут.",
        "Идите вперёд и не перечитывайте. Текст выше приглушён нарочно: пусть написанное не тянет назад.",
        "Застряли — так и напишите: «застрял, потому что…». Честная строка всегда лучше пустой.",
        "Не нашли слово или факт — оставьте пропуск и бегите дальше. Разберётесь на правке, не сейчас.",
        "О сохранении не думайте: текст ложится на диск сам, едва вы перестали печатать.",
        "Заход окончен — останавливайтесь хоть на полуслове. Дописали черновик целиком — вам в «Правлю».",
    ],
};

/// The mode where the goal flips (CONCEPT.md, механика 3).
static EDIT: Method = Method {
    lead: "Теперь можно быть строгим. Текст перед вами целиком — и всё лишнее в нём наконец видно.",
    steps: &[
        "Сначала прочтите всё подряд, не трогая ни слова. Просто заметьте, где вам стало скучно.",
        "Смело режьте крупное. Абзац, который не работает, лечится удалением, а не починкой.",
        "Переставьте оставшееся: сильное — в начало и в конец. Середину читают вполглаза.",
        "Закройте пропуски из черновика и только потом беритесь за фразы: короче, точнее, живее.",
        "Захотелось дописать целый кусок — вернитесь в «Пишу». Не смешивайте: это две разные работы.",
        "Заголовок — в самом конце, когда видно, о чём получилось. И заканчивайте: дальше публикация.",
    ],
};

/// The only two endings, and why the shelved one is not a failure
/// (CONCEPT.md, механика 5).
static FINISHING: Method = Method {
    lead: "Самый страшный шаг и самый важный. Текст оживает, только когда его прочли, — иначе он навсегда черновик.",
    steps: &[
        "«Ещё не готово» — это не про текст, а про страх. Сам он не пройдёт: решайте, отдаёте ли вы текст читателю.",
        "Скопируйте как markdown или сохраните в файл — уйдут только ваши слова, без служебных строк.",
        "Опубликуйте по-настоящему: блог, рассылка, канал. Место, где текст встретит живого человека.",
        "Вернитесь и вставьте ссылку. Необязательно — но потом вы увидите, что всё это было всерьёз.",
        "Подтвердите. Эссе встанет в ряд опубликованных, а руки освободятся для следующего.",
        "Текст правда не тот — отправьте «в стол». Это взрослое решение, а не поражение; но вернуть его нельзя.",
    ],
};

/// The conveyor seen whole, and the rule that keeps it moving (CONCEPT.md,
/// «Конвейер»).
static SHELF: Method = Method {
    lead: "Отсюда видно всё сразу: что ждёт своей очереди, что в работе и что уже дошло до людей.",
    steps: &[
        "Слева копилка искр, свежие сверху. Пока она не пуста, «не о чем писать» вам больше не грозит.",
        "В середине то, что в работе. Оно всегда одно: одно эссе доходит до конца чаще, чем пять начатых.",
        "Справа опубликованное, с датами и ссылками. Вот это и есть ваш прогресс, а не счётчик слов.",
        "Внизу «стол» — отложенное осознанно. Перечитать можно, вернуть в работу нет, и это честно.",
        "Тянет к другому — доведите текущее до конца. Свободные руки здесь зарабатываются, а не выдаются.",
    ],
};

/// Wider than the shortcut sheet, because this is prose and prose wants a
/// measure. Narrow enough to still fit the Today window, which is the smallest
/// the app ever gets.
const CARD_WIDTH: f32 = 560.;

/// The same veil as the shortcut sheet: the two are one family, and the work
/// stays legible under both.
const VEIL: u32 = 0x15171baa;

#[cfg(test)]
mod tests {
    use super::*;

    const PLACES: [Place; 6] = [
        Place::Today,
        Place::ChoosingSpark,
        Place::Write,
        Place::Edit,
        Place::Finishing,
        Place::Shelf,
    ];

    /// Every place the writer can stand in gets a whole instruction: what it is
    /// for, and enough steps to actually work through it (writing-guidance
    /// spec).
    #[test]
    fn every_place_is_explained_step_by_step() {
        for place in PLACES {
            let method = method(place);
            assert!(!method.lead.trim().is_empty(), "{place:?} has no lead");
            assert!(
                method.steps.len() >= 3,
                "{place:?} has {} step(s) — too few to be an instruction",
                method.steps.len()
            );
            for step in method.steps {
                assert!(!step.trim().is_empty(), "{place:?} has a blank step");
            }
        }
    }

    /// The sheet has to fit on screen whole — there is no scrolling in it,
    /// because any key dismisses it (design.md, D2). Both dimensions are
    /// capped: how many steps, and how long each one wraps to. The budget is
    /// in characters, counted as characters — every one of these is two bytes.
    #[test]
    fn no_place_outgrows_the_sheet() {
        // Two wrapped lines at the card's measure, with the number in front.
        const LONGEST_STEP: usize = 120;

        for place in PLACES {
            let method = method(place);
            assert!(
                method.steps.len() <= 6,
                "{place:?} has {} steps — more than the sheet can show at once",
                method.steps.len()
            );
            assert!(
                method.lead.chars().count() <= LONGEST_STEP,
                "{place:?} opens with a paragraph, not a line"
            );
            for step in method.steps {
                let length = step.chars().count();
                assert!(
                    length <= LONGEST_STEP,
                    "a step in {place:?} is {length} characters — too long to stay on two lines: {step}"
                );
            }
        }
    }

    /// Each place says its own thing: two identical texts would mean the sheet
    /// stopped following where the writer is standing.
    #[test]
    fn every_place_says_its_own_thing() {
        for (index, place) in PLACES.iter().enumerate() {
            for other in &PLACES[index + 1..] {
                assert_ne!(
                    method(*place).lead,
                    method(*other).lead,
                    "{place:?} and {other:?} open the same way"
                );
                assert_ne!(
                    method(*place).steps,
                    method(*other).steps,
                    "{place:?} and {other:?} give the same steps"
                );
            }
        }
    }
}
