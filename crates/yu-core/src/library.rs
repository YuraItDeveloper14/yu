//! The library: every word of Yu, what it does and a short example, in Ukrainian and English.
//! Yu Studio shows it in its sidebar; the tests below run every example.

use crate::builtins::Builtin;
use crate::keywords::Keyword;
use crate::lang::Lang;

/// A part of the library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Basics,
    Conditions,
    Loops,
    Functions,
    Lists,
    Drawing,
    Turtle,
}

impl Section {
    /// Every section, in the order the library shows them.
    pub const ALL: [Section; 7] = [
        Section::Basics,
        Section::Conditions,
        Section::Loops,
        Section::Functions,
        Section::Lists,
        Section::Drawing,
        Section::Turtle,
    ];

    /// A short name for the page, e.g. `loops`.
    pub fn id(self) -> &'static str {
        match self {
            Section::Basics => "basics",
            Section::Conditions => "conditions",
            Section::Loops => "loops",
            Section::Functions => "functions",
            Section::Lists => "lists",
            Section::Drawing => "drawing",
            Section::Turtle => "turtle",
        }
    }

    pub fn title(self, lang: Lang) -> &'static str {
        match self {
            Section::Basics => lang.pick("Основи", "Basics"),
            Section::Conditions => lang.pick("Умови", "Conditions"),
            Section::Loops => lang.pick("Цикли", "Loops"),
            Section::Functions => lang.pick("Функції", "Functions"),
            Section::Lists => lang.pick("Списки", "Lists"),
            Section::Drawing => lang.pick("Малювання", "Drawing"),
            Section::Turtle => lang.pick("Черепашка", "Turtle"),
        }
    }
}

/// What an entry explains: keywords that work together, or one built-in.
#[derive(Debug, Clone, Copy)]
pub enum Word {
    /// `name` and `form` are Ukrainian first, then English.
    Keywords {
        words: &'static [Keyword],
        name: [&'static str; 2],
        form: [&'static str; 2],
    },
    Builtin(Builtin),
}

/// One entry of the library; its text and example are Ukrainian first, then English.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    pub section: Section,
    pub word: Word,
    text: [&'static str; 2],
    example: [&'static str; 2],
}

impl Entry {
    /// The word as the library shows it: `коло` or `повтори … разів`.
    pub fn name(&self, lang: Lang) -> &'static str {
        match self.word {
            Word::Keywords { name, .. } => lang.pick(name[0], name[1]),
            Word::Builtin(b) => b.name(lang),
        }
    }

    /// How to write it: `коло(x, y, радіус)` or `повтори N разів:`.
    pub fn form(&self, lang: Lang) -> &'static str {
        match self.word {
            Word::Keywords { form, .. } => lang.pick(form[0], form[1]),
            Word::Builtin(b) => b.signature(lang),
        }
    }

    /// What it does, in a sentence or two.
    pub fn text(&self, lang: Lang) -> &'static str {
        lang.pick(self.text[0], self.text[1])
    }

    /// A short program that uses it.
    pub fn example(&self, lang: Lang) -> &'static str {
        lang.pick(self.example[0], self.example[1])
    }
}

const fn builtin(
    section: Section,
    b: Builtin,
    text: [&'static str; 2],
    example: [&'static str; 2],
) -> Entry {
    Entry {
        section,
        word: Word::Builtin(b),
        text,
        example,
    }
}

const fn keywords(
    section: Section,
    words: &'static [Keyword],
    name: [&'static str; 2],
    form: [&'static str; 2],
    text: [&'static str; 2],
    example: [&'static str; 2],
) -> Entry {
    Entry {
        section,
        word: Word::Keywords { words, name, form },
        text,
        example,
    }
}

/// Every entry, section by section.
pub const ENTRIES: [Entry; 37] = [
    builtin(
        Section::Basics,
        Builtin::Say,
        [
            "Друкує в терміналі те, що в дужках. Кілька значень через кому стають одним рядком, між ними пробіл.",
            "Prints what is in the brackets in the terminal. Several values separated by commas become one line with spaces between them.",
        ],
        [
            "скажи(\"Привіт, світе!\")\nскажи(\"2 + 2 =\", 2 + 2)",
            "say(\"Hello, world!\")\nsay(\"2 + 2 =\", 2 + 2)",
        ],
    ),
    builtin(
        Section::Basics,
        Builtin::Ask,
        [
            "Показує питання й чекає, поки людина введе відповідь. Відповідь завжди текст; щоб отримати число, загорни її в число(…).",
            "Shows a question and waits for the person to type an answer. The answer is always text; wrap it in number(…) to get a number.",
        ],
        [
            "ім'я = запитай(\"Як тебе звати?\")\nскажи(\"Привіт, \" + ім'я + \"!\")",
            "name = ask(\"What is your name?\")\nsay(\"Hi, \" + name + \"!\")",
        ],
    ),
    builtin(
        Section::Basics,
        Builtin::Text,
        [
            "Перетворює значення на текст. Так можна, наприклад, дізнатися, скільки цифр у числі.",
            "Turns a value into text. That way you can, for example, count the digits of a number.",
        ],
        [
            "рік = 2026\nскажи(\"У числі\", рік, \"цифр:\", довжина(текст(рік)))",
            "year = 2026\nsay(\"The number\", year, \"has\", length(text(year)), \"digits\")",
        ],
    ),
    builtin(
        Section::Basics,
        Builtin::Number,
        [
            "Перетворює текст на число, щоб із ним можна було рахувати. Десяткову частину можна відділити крапкою або комою.",
            "Turns text into a number you can calculate with. A decimal point or a comma both work.",
        ],
        [
            "ціна = число(\"12,5\")\nскажи(\"Дві штуки коштують\", ціна * 2)",
            "price = number(\"12.5\")\nsay(\"Two of them cost\", price * 2)",
        ],
    ),
    builtin(
        Section::Basics,
        Builtin::Round,
        [
            "Округлює число до найближчого цілого: 2.4 стане 2, а 2.5 — 3.",
            "Rounds a number to the nearest whole number: 2.4 becomes 2 and 2.5 becomes 3.",
        ],
        [
            "середнє = (7 + 8 + 10) / 3\nскажи(\"Середня оцінка:\", округли(середнє))",
            "average = (7 + 8 + 10) / 3\nsay(\"Average mark:\", round(average))",
        ],
    ),
    builtin(
        Section::Basics,
        Builtin::Random,
        [
            "Дає випадкове ціле число від a до b, обидва кінці включно. Щоразу інше.",
            "Gives a random whole number from a to b, both ends included. It is different every time.",
        ],
        [
            "кубик = випадкове(1, 6)\nскажи(\"На кубику випало\", кубик)",
            "dice = random(1, 6)\nsay(\"The dice shows\", dice)",
        ],
    ),
    keywords(
        Section::Conditions,
        &[Keyword::If],
        ["якщо", "if"],
        ["якщо умова:", "if condition:"],
        [
            "Виконує блок під собою, лише коли умова правдива. Блок — це рядки з відступом у чотири пробіли.",
            "Runs the block below it only when the condition is true. The block is the lines indented by four spaces.",
        ],
        [
            "температура = 25\nякщо температура > 20:\n    скажи(\"Сьогодні тепло\")",
            "temperature = 25\nif temperature > 20:\n    say(\"It is warm today\")",
        ],
    ),
    keywords(
        Section::Conditions,
        &[Keyword::Else],
        ["інакше", "else"],
        ["інакше: · інакше якщо умова:", "else: · else if condition:"],
        [
            "Іде після якщо. Блок під інакше виконується, коли умова неправдива, а інакше якщо перевіряє ще одну умову.",
            "Comes after if. The block under else runs when the condition is false, and else if checks one more condition.",
        ],
        [
            "бал = 7\nякщо бал >= 10:\n    скажи(\"Відмінно\")\nінакше якщо бал >= 6:\n    скажи(\"Добре\")\nінакше:\n    скажи(\"Спробуй ще\")",
            "score = 7\nif score >= 10:\n    say(\"Excellent\")\nelse if score >= 6:\n    say(\"Good\")\nelse:\n    say(\"Try again\")",
        ],
    ),
    keywords(
        Section::Conditions,
        &[Keyword::And, Keyword::Or, Keyword::Not],
        ["і · або · не", "and · or · not"],
        ["a і b · a або b · не a", "a and b · a or b · not a"],
        [
            "Поєднують умови. «і» правдиве, коли правдиві обидві частини, «або» — коли хоч одна, а «не» перевертає так на ні й навпаки.",
            "Combine conditions. “and” is true when both parts are true, “or” when at least one is, and “not” turns true into false and back.",
        ],
        [
            "вік = 12\nякщо вік > 6 і вік < 18:\n    скажи(\"Ходить до школи\")\nякщо вік < 6 або вік > 65:\n    скажи(\"Не ходить до школи\")\nскажи(\"Дорослий:\", не (вік < 18))",
            "age = 12\nif age > 6 and age < 18:\n    say(\"Goes to school\")\nif age < 6 or age > 65:\n    say(\"Does not go to school\")\nsay(\"Grown up:\", not (age < 18))",
        ],
    ),
    keywords(
        Section::Conditions,
        &[Keyword::True, Keyword::False],
        ["так · ні", "true · false"],
        ["так · ні", "true · false"],
        [
            "Два значення правди. Їх дають порівняння, а якщо й поки за ними вирішують, що робити.",
            "The two truth values. Comparisons give them, and if and while use them to decide what to do.",
        ],
        [
            "сонце = так\nдощ = ні\nскажи(\"Сонце:\", сонце, \"Дощ:\", дощ)\nскажи(\"5 > 3:\", 5 > 3)",
            "sun = true\nrain = false\nsay(\"Sun:\", sun, \"Rain:\", rain)\nsay(\"5 > 3:\", 5 > 3)",
        ],
    ),
    keywords(
        Section::Conditions,
        &[Keyword::Nothing],
        ["нічого", "nothing"],
        ["нічого", "nothing"],
        [
            "Означає, що значення немає. Його повертає функція, у якій немає поверни.",
            "Means there is no value. A function without return gives it back.",
        ],
        [
            "знахідка = нічого\nскажи(\"Знайшли:\", знахідка)",
            "found = nothing\nsay(\"Found:\", found)",
        ],
    ),
    keywords(
        Section::Loops,
        &[Keyword::Repeat, Keyword::Times],
        ["повтори … разів", "repeat … times"],
        ["повтори N разів:", "repeat N times:"],
        [
            "Повторює блок N разів. Після числа пиши раз, рази чи разів, як звучить природно: повтори 1 раз, повтори 3 рази, повтори 5 разів.",
            "Repeats the block N times. The word times after the number may be left out.",
        ],
        [
            "зірочки = \"\"\nповтори 5 разів:\n    зірочки = зірочки + \"*\"\nскажи(зірочки)",
            "stars = \"\"\nrepeat 5 times:\n    stars = stars + \"*\"\nsay(stars)",
        ],
    ),
    keywords(
        Section::Loops,
        &[Keyword::While],
        ["поки", "while"],
        ["поки умова:", "while condition:"],
        [
            "Повторює блок, поки умова правдива. Стеж, щоб умова колись стала неправдивою, бо інакше цикл не закінчиться.",
            "Repeats the block while the condition is true. Make sure the condition turns false at some point, or the loop never ends.",
        ],
        [
            "гроші = 10\nпоки гроші >= 3:\n    гроші = гроші - 3\n    скажи(\"Купив морозиво, лишилось\", гроші)",
            "money = 10\nwhile money >= 3:\n    money = money - 3\n    say(\"Bought an ice cream,\", money, \"left\")",
        ],
    ),
    keywords(
        Section::Loops,
        &[Keyword::For, Keyword::From, Keyword::To, Keyword::Step],
        ["для … від … до", "for … from … to"],
        ["для i від A до B крок S:", "for i from A to B step S:"],
        [
            "Рахує від A до B, обидва кінці включно, і для кожного числа виконує блок. Крок можна не писати, тоді він 1.",
            "Counts from A to B, both ends included, and runs the block for each number. The step may be left out; then it is 1.",
        ],
        [
            "для i від 1 до 3:\n    скажи(i, \"× 2 =\", i * 2)\nдля i від 0 до 20 крок 10:\n    скажи(i)",
            "for i from 1 to 3:\n    say(i, \"× 2 =\", i * 2)\nfor i from 0 to 20 step 10:\n    say(i)",
        ],
    ),
    keywords(
        Section::Loops,
        &[Keyword::For, Keyword::In],
        ["для … у", "for … in"],
        ["для x у список:", "for x in list:"],
        [
            "Проходить список по одному елементу: щоразу x — наступний елемент. Замість у можна писати в.",
            "Goes through a list one item at a time: each time x is the next item.",
        ],
        [
            "для фрукт у [\"яблуко\", \"груша\", \"слива\"]:\n    скажи(\"У кошику:\", фрукт)",
            "for fruit in [\"apple\", \"pear\", \"plum\"]:\n    say(\"In the basket:\", fruit)",
        ],
    ),
    keywords(
        Section::Loops,
        &[Keyword::Break],
        ["стоп", "break"],
        ["стоп", "break"],
        [
            "Одразу виходить із циклу, навіть якщо він ще не закінчився.",
            "Leaves the loop at once, even if it has not finished.",
        ],
        [
            "для i від 1 до 10:\n    якщо i == 4:\n        стоп\n    скажи(i)",
            "for i from 1 to 10:\n    if i == 4:\n        break\n    say(i)",
        ],
    ),
    keywords(
        Section::Loops,
        &[Keyword::Continue],
        ["далі", "continue"],
        ["далі", "continue"],
        [
            "Пропускає решту блоку й одразу переходить до наступного повтору циклу.",
            "Skips the rest of the block and goes straight to the next round of the loop.",
        ],
        [
            "для i від 1 до 6:\n    якщо i % 2 == 0:\n        далі\n    скажи(i, \"— непарне\")",
            "for i from 1 to 6:\n    if i % 2 == 0:\n        continue\n    say(i, \"is odd\")",
        ],
    ),
    keywords(
        Section::Functions,
        &[Keyword::Function],
        ["функція", "function"],
        ["функція назва(параметри):", "function name(parameters):"],
        [
            "Дає блоку назву, щоб викликати його знову й знову. У дужках — параметри: значення, які функція отримує під час виклику.",
            "Gives a block a name so you can call it again and again. The brackets hold the parameters: values the function gets when it is called.",
        ],
        [
            "функція привітай(ім'я):\n    скажи(\"Привіт, \" + ім'я)\n\nпривітай(\"Оля\")\nпривітай(\"Тарас\")",
            "function greet(name):\n    say(\"Hi, \" + name)\n\ngreet(\"Olya\")\ngreet(\"Taras\")",
        ],
    ),
    keywords(
        Section::Functions,
        &[Keyword::Return],
        ["поверни", "return"],
        ["поверни значення", "return value"],
        [
            "Закінчує функцію й віддає значення туди, де її викликали.",
            "Ends the function and hands a value back to where it was called.",
        ],
        [
            "функція квадрат(n):\n    поверни n * n\n\nскажи(\"5 у квадраті:\", квадрат(5))",
            "function square(n):\n    return n * n\n\nsay(\"5 squared:\", square(5))",
        ],
    ),
    builtin(
        Section::Lists,
        Builtin::Length,
        [
            "Скільки елементів у списку або символів у тексті.",
            "How many items a list has, or how many characters a text has.",
        ],
        [
            "друзі = [\"Оля\", \"Тарас\", \"Марко\"]\nскажи(\"Друзів:\", довжина(друзі))\nскажи(\"Літер у слові Київ:\", довжина(\"Київ\"))",
            "friends = [\"Olya\", \"Taras\", \"Marko\"]\nsay(\"Friends:\", length(friends))\nsay(\"Letters in Kyiv:\", length(\"Kyiv\"))",
        ],
    ),
    builtin(
        Section::Lists,
        Builtin::Append,
        [
            "Додає значення в кінець списку. Елементи нумеруються з 1: список[1] — перший.",
            "Adds a value to the end of a list. Items are numbered from 1: list[1] is the first.",
        ],
        [
            "покупки = [\"хліб\"]\nдодай(покупки, \"молоко\")\nскажи(покупки)\nскажи(\"Друге:\", покупки[2])",
            "shopping = [\"bread\"]\nappend(shopping, \"milk\")\nsay(shopping)\nsay(\"Second:\", shopping[2])",
        ],
    ),
    builtin(
        Section::Drawing,
        Builtin::Canvas,
        [
            "Змінює розмір полотна в пікселях. Спочатку воно 600 на 400; викликай полотно на самому початку програми.",
            "Changes the size of the canvas in pixels. It starts at 600 by 400; call canvas at the very beginning of the program.",
        ],
        [
            "полотно(300, 300)\nфон(\"блакитний\")\nколір(\"жовтий\")\nколо(150, 150, 100)",
            "canvas(300, 300)\nbackground(\"lightblue\")\ncolor(\"yellow\")\ncircle(150, 150, 100)",
        ],
    ),
    builtin(
        Section::Drawing,
        Builtin::Background,
        [
            "Зафарбовує все полотно одним кольором. Назву кольору пиши в лапках.",
            "Paints the whole canvas one colour. Write the colour's name in quotes.",
        ],
        [
            "фон(\"синій\")\nколір(\"жовтий\")\nпрямокутник(0, 200, 600, 200)",
            "background(\"blue\")\ncolor(\"yellow\")\nrect(0, 200, 600, 200)",
        ],
    ),
    builtin(
        Section::Drawing,
        Builtin::Color,
        [
            "Задає колір наступних фігур, ліній і написів. Підходить назва в будь-якому роді, \"#ff8800\" або \"випадковий\".",
            "Sets the colour of the next shapes, lines and labels. A colour name, \"#ff8800\" or \"random\" all work.",
        ],
        [
            "колір(\"червона\")\nколо(200, 200, 80)\nколір(\"#ff8800\")\nколо(400, 200, 80)",
            "color(\"red\")\ncircle(200, 200, 80)\ncolor(\"#ff8800\")\ncircle(400, 200, 80)",
        ],
    ),
    builtin(
        Section::Drawing,
        Builtin::Thickness,
        [
            "Задає товщину ліній у пікселях: і для лінії, і для черепашки.",
            "Sets the width of lines in pixels, both for line and for the turtle.",
        ],
        [
            "товщина(2)\nлінія(100, 150, 500, 150)\nтовщина(12)\nлінія(100, 250, 500, 250)",
            "thickness(2)\nline(100, 150, 500, 150)\nthickness(12)\nline(100, 250, 500, 250)",
        ],
    ),
    builtin(
        Section::Drawing,
        Builtin::Circle,
        [
            "Малює зафарбоване коло: x і y — центр, радіус — відстань від центру до краю.",
            "Draws a filled circle: x and y are the centre, r is the distance from the centre to the edge.",
        ],
        [
            "колір(\"червоний\")\nколо(300, 200, 80)",
            "color(\"red\")\ncircle(300, 200, 80)",
        ],
    ),
    builtin(
        Section::Drawing,
        Builtin::Rect,
        [
            "Малює зафарбований прямокутник: x і y — лівий верхній кут, далі ширина й висота.",
            "Draws a filled rectangle: x and y are the top left corner, then the width and the height.",
        ],
        [
            "колір(\"коричневий\")\nпрямокутник(200, 150, 200, 150)\nколір(\"червоний\")\nпрямокутник(270, 220, 60, 80)",
            "color(\"brown\")\nrect(200, 150, 200, 150)\ncolor(\"red\")\nrect(270, 220, 60, 80)",
        ],
    ),
    builtin(
        Section::Drawing,
        Builtin::Line,
        [
            "Малює пряму лінію від точки x1, y1 до точки x2, y2.",
            "Draws a straight line from the point x1, y1 to the point x2, y2.",
        ],
        [
            "колір(\"зелений\")\nтовщина(6)\nлінія(100, 300, 300, 100)\nлінія(300, 100, 500, 300)",
            "color(\"green\")\nthickness(6)\nline(100, 300, 300, 100)\nline(300, 100, 500, 300)",
        ],
    ),
    builtin(
        Section::Drawing,
        Builtin::Label,
        [
            "Пише текст на полотні; x і y — лівий верхній кут напису.",
            "Writes text on the canvas; x and y are the top left corner of the text.",
        ],
        [
            "колір(\"синій\")\nнапис(\"Привіт від Yu\", 220, 190)",
            "color(\"blue\")\nlabel(\"Hello from Yu\", 220, 190)",
        ],
    ),
    builtin(
        Section::Turtle,
        Builtin::Forward,
        [
            "Черепашка йде вперед на n пікселів і малює лінію, якщо перо опущене. Вона починає в центрі й дивиться праворуч.",
            "The turtle walks n pixels ahead and draws a line while the pen is down. It starts in the centre, facing right.",
        ],
        [
            "колір(\"синій\")\nвперед(150)",
            "color(\"blue\")\nforward(150)",
        ],
    ),
    builtin(
        Section::Turtle,
        Builtin::Back,
        [
            "Черепашка йде назад на n пікселів, не розвертаючись.",
            "The turtle walks n pixels backwards without turning around.",
        ],
        [
            "вперед(100)\nназад(200)",
            "forward(100)\nback(200)",
        ],
    ),
    builtin(
        Section::Turtle,
        Builtin::Right,
        [
            "Черепашка повертає праворуч, за годинниковою стрілкою, на стільки градусів.",
            "The turtle turns right, clockwise, by that many degrees.",
        ],
        [
            "повтори 4 рази:\n    вперед(120)\n    праворуч(90)",
            "repeat 4 times:\n    forward(120)\n    right(90)",
        ],
    ),
    builtin(
        Section::Turtle,
        Builtin::Left,
        [
            "Черепашка повертає ліворуч, проти годинникової стрілки, на стільки градусів.",
            "The turtle turns left, anticlockwise, by that many degrees.",
        ],
        [
            "повтори 3 рази:\n    вперед(150)\n    ліворуч(120)",
            "repeat 3 times:\n    forward(150)\n    left(120)",
        ],
    ),
    builtin(
        Section::Turtle,
        Builtin::PenUp,
        [
            "Черепашка далі ходить, але не малює, доки не опустиш перо.",
            "The turtle keeps walking but stops drawing until you put the pen down.",
        ],
        [
            "вперед(60)\nпідніми_перо()\nвперед(40)\nопусти_перо()\nвперед(60)",
            "forward(60)\npen_up()\nforward(40)\npen_down()\nforward(60)",
        ],
    ),
    builtin(
        Section::Turtle,
        Builtin::PenDown,
        [
            "Черепашка знову малює, коли йде. На початку перо вже опущене.",
            "The turtle draws again as it walks. At the start the pen is already down.",
        ],
        [
            "підніми_перо()\nназад(200)\nопусти_перо()\nвперед(400)",
            "pen_up()\nback(200)\npen_down()\nforward(400)",
        ],
    ),
    builtin(
        Section::Turtle,
        Builtin::BeginFill,
        [
            "Починає запам'ятовувати шлях черепашки, щоб потім зафарбувати фігуру всередині.",
            "Starts recording the turtle's path so the shape inside can be filled later.",
        ],
        [
            "колір(\"жовтий\")\nпочни_заливку()\nповтори 5 разів:\n    вперед(200)\n    праворуч(144)\nзаверши_заливку()",
            "color(\"yellow\")\nbegin_fill()\nrepeat 5 times:\n    forward(200)\n    right(144)\nend_fill()",
        ],
    ),
    builtin(
        Section::Turtle,
        Builtin::EndFill,
        [
            "Зафарбовує поточним кольором фігуру, яку черепашка обійшла після почни_заливку.",
            "Fills the shape the turtle walked around since begin_fill with the current colour.",
        ],
        [
            "колір(\"зелений\")\nпочни_заливку()\nповтори 3 рази:\n    вперед(160)\n    ліворуч(120)\nзаверши_заливку()",
            "color(\"green\")\nbegin_fill()\nrepeat 3 times:\n    forward(160)\n    left(120)\nend_fill()",
        ],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::NAMES;
    use crate::keywords::SPELLINGS;
    use crate::lexer::lex;
    use crate::token::TokenKind;
    use crate::{Host, Options, Session};

    /// Keeps what an example prints and answers every question with «Юрій».
    #[derive(Default)]
    struct Sample {
        out: Vec<String>,
    }

    impl Host for Sample {
        fn print(&mut self, line: &str) {
            self.out.push(line.to_string());
        }
        fn ask(&mut self, _prompt: &str) -> Option<String> {
            Some("Юрій".to_string())
        }
    }

    const LANGS: [Lang; 2] = [Lang::Uk, Lang::En];

    #[test]
    fn every_word_of_yu_has_an_entry() {
        for (b, uk, _) in NAMES {
            let entries = ENTRIES
                .iter()
                .filter(|e| matches!(e.word, Word::Builtin(x) if x == b))
                .count();
            assert_eq!(entries, 1, "{uk}");
        }
        for (spelling, k) in SPELLINGS {
            let covered = ENTRIES
                .iter()
                .any(|e| matches!(e.word, Word::Keywords { words, .. } if words.contains(&k)));
            assert!(covered, "{spelling}");
        }
    }

    #[test]
    fn entries_come_section_by_section() {
        let order: Vec<usize> = ENTRIES
            .iter()
            .map(|e| Section::ALL.iter().position(|s| *s == e.section).unwrap())
            .collect();
        assert!(order.windows(2).all(|pair| pair[0] <= pair[1]));
        for section in Section::ALL {
            assert!(ENTRIES.iter().any(|e| e.section == section), "{section:?}");
        }
    }

    #[test]
    fn entries_read_well() {
        let circle = ENTRIES
            .iter()
            .find(|e| matches!(e.word, Word::Builtin(Builtin::Circle)))
            .unwrap();
        assert_eq!(circle.name(Lang::Uk), "коло");
        assert_eq!(circle.form(Lang::En), "circle(x, y, r)");
        assert_eq!(
            circle.example(Lang::Uk),
            "колір(\"червоний\")\nколо(300, 200, 80)"
        );
        let repeat = ENTRIES
            .iter()
            .find(|e| e.name(Lang::En) == "repeat … times")
            .unwrap();
        assert_eq!(repeat.form(Lang::Uk), "повтори N разів:");
        assert_eq!(repeat.section.title(Lang::Uk), "Цикли");
        assert_eq!(repeat.section.id(), "loops");
        for entry in &ENTRIES {
            for lang in LANGS {
                let lines = entry.example(lang).lines().count();
                assert!(
                    (2..=7).contains(&lines),
                    "{}: {lines} lines",
                    entry.name(lang)
                );
                assert!(!entry.text(lang).is_empty(), "{}", entry.name(lang));
            }
        }
    }

    #[test]
    fn every_example_runs_and_shows_something() {
        for entry in &ENTRIES {
            for lang in LANGS {
                let example = entry.example(lang);
                let mut session = Session::new(Options {
                    lang,
                    step_budget: Some(100_000),
                    ..Options::default()
                });
                let mut host = Sample::default();
                if let Err(e) = session.run(example, &mut host) {
                    panic!("{}:\n{}", entry.name(lang), e.render(example, lang));
                }
                if matches!(entry.section, Section::Drawing | Section::Turtle) {
                    assert!(
                        !session.drawing().is_empty(),
                        "{} draws nothing",
                        entry.name(lang)
                    );
                } else {
                    assert!(!host.out.is_empty(), "{} prints nothing", entry.name(lang));
                }
            }
        }
    }

    #[test]
    fn every_example_uses_its_words_and_speaks_one_language() {
        for entry in &ENTRIES {
            for lang in LANGS {
                let example = entry.example(lang);
                let tokens = lex(example).expect("an example lexes");
                for token in &tokens {
                    match &token.kind {
                        TokenKind::Kw(_) => {
                            let spelled = &example[token.span.start..token.span.end];
                            assert_eq!(
                                spelled.is_ascii(),
                                lang == Lang::En,
                                "{} uses «{spelled}»",
                                entry.name(lang)
                            );
                        }
                        TokenKind::Name(name) => {
                            if let Some(b) = Builtin::lookup(name) {
                                assert_eq!(b.name(lang), name.as_str(), "{}", entry.name(lang));
                            }
                        }
                        _ => {}
                    }
                }
                match entry.word {
                    Word::Builtin(b) => {
                        let called = TokenKind::Name(b.name(lang).to_string());
                        assert!(
                            tokens.iter().any(|t| t.kind == called),
                            "{}",
                            entry.name(lang)
                        );
                    }
                    Word::Keywords { words, .. } => {
                        for k in words {
                            assert!(
                                tokens.iter().any(|t| t.kind == TokenKind::Kw(*k)),
                                "{} lacks {k:?}",
                                entry.name(lang)
                            );
                        }
                    }
                }
            }
        }
    }
}
