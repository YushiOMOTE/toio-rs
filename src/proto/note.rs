use anyhow::{bail, Error, Result};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::convert::TryFrom;

/// Sound note.
#[derive(
    Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[repr(u8)]
pub enum Note {
    /// C 0
    C0 = 0,
    /// C# 0
    CS0,
    /// D 0
    D0,
    /// D# 0
    DS0,
    /// E 0
    E0,
    /// F 0
    F0,
    /// F# 0
    FS0,
    /// G 0
    G0,
    /// G# 0
    GS0,
    /// A 0
    A0,
    /// A# 0
    AS0,
    /// B 0
    B0,
    /// C 1
    C1,
    /// C# 1
    CS1,
    /// D 1
    D1,
    /// D# 1
    DS1,
    /// E 1
    E1,
    /// F 1
    F1,
    /// F# 1
    FS1,
    /// G 1
    G1,
    /// G# 1
    GS1,
    /// A 1
    A1,
    /// A# 1
    AS1,
    /// B 1
    B1,
    /// C 2
    C2,
    /// C# 2
    CS2,
    /// D 2
    D2,
    /// D# 2
    DS2,
    /// E 2
    E2,
    /// F 2
    F2,
    /// F# 2
    FS2,
    /// G 2
    G2,
    /// G# 2
    GS2,
    /// A 2
    A2,
    /// A# 2
    AS2,
    /// B 2
    B2,
    /// C 3
    C3,
    /// C# 3
    CS3,
    /// D 3
    D3,
    /// D# 3
    DS3,
    /// E 3
    E3,
    /// F 3
    F3,
    /// F# 3
    FS3,
    /// G 3
    G3,
    /// G# 3
    GS3,
    /// A 3
    A3,
    /// A# 3
    AS3,
    /// B 3
    B3,
    /// C 4
    C4,
    /// C# 4
    CS4,
    /// D 4
    D4,
    /// D# 4
    DS4,
    /// E 4
    E4,
    /// F 4
    F4,
    /// F# 4
    FS4,
    /// G 4
    G4,
    /// G# 4
    GS4,
    /// A 4
    A4,
    /// A# 4
    AS4,
    /// B 4
    B4,
    /// C 5
    C5,
    /// C# 5
    CS5,
    /// D 5
    D5,
    /// D# 5
    DS5,
    /// E 5
    E5,
    /// F 5
    F5,
    /// F# 5
    FS5,
    /// G 5
    G5,
    /// G# 5
    GS5,
    /// A 5
    A5,
    /// A# 5
    AS5,
    /// B 5
    B5,
    /// C 6
    C6,
    /// C# 6
    CS6,
    /// D 6
    D6,
    /// D# 6
    DS6,
    /// E 6
    E6,
    /// F 6
    F6,
    /// F# 6
    FS6,
    /// G 6
    G6,
    /// G# 6
    GS6,
    /// A 6
    A6,
    /// A# 6
    AS6,
    /// B 6
    B6,
    /// C 7
    C7,
    /// C# 7
    CS7,
    /// D 7
    D7,
    /// D# 7
    DS7,
    /// E 7
    E7,
    /// F 7
    F7,
    /// F# 7
    FS7,
    /// G 7
    G7,
    /// G# 7
    GS7,
    /// A 7
    A7,
    /// A# 7
    AS7,
    /// B 7
    B7,
    /// C 8
    C8,
    /// C# 8
    CS8,
    /// D 8
    D8,
    /// D# 8
    DS8,
    /// E 8
    E8,
    /// F 8
    F8,
    /// F# 8
    FS8,
    /// G 8
    G8,
    /// G# 8
    GS8,
    /// A 8
    A8,
    /// A# 8
    AS8,
    /// B 8
    B8,
    /// C 9
    C9,
    /// C# 9
    CS9,
    /// D 9
    D9,
    /// D# 9
    DS9,
    /// E 9
    E9,
    /// F 9
    F9,
    /// F# 9
    FS9,
    /// G 9
    G9,
    /// G# 9
    GS9,
    /// A 9
    A9,
    /// A# 9
    AS9,
    /// B 9
    B9,
    /// C 10
    C10,
    /// C# 10
    CS10,
    /// D 10
    D10,
    /// D# 10
    DS10,
    /// E 10
    E10,
    /// F 10
    F10,
    /// F# 10
    FS10,
    /// G 10
    G10,
    /// No sound.
    NoSound,
}

impl From<Note> for u8 {
    fn from(note: Note) -> Self {
        note as u8
    }
}

impl TryFrom<u8> for Note {
    type Error = Error;

    fn try_from(note: u8) -> Result<Self> {
        use Note::*;

        let note = match note {
            0 => C0,
            1 => CS0,
            2 => D0,
            3 => DS0,
            4 => E0,
            5 => F0,
            6 => FS0,
            7 => G0,
            8 => GS0,
            9 => A0,
            10 => AS0,
            11 => B0,
            12 => C1,
            13 => CS1,
            14 => D1,
            15 => DS1,
            16 => E1,
            17 => F1,
            18 => FS1,
            19 => G1,
            20 => GS1,
            21 => A1,
            22 => AS1,
            23 => B1,
            24 => C2,
            25 => CS2,
            26 => D2,
            27 => DS2,
            28 => E2,
            29 => F2,
            30 => FS2,
            31 => G2,
            32 => GS2,
            33 => A2,
            34 => AS2,
            35 => B2,
            36 => C3,
            37 => CS3,
            38 => D3,
            39 => DS3,
            40 => E3,
            41 => F3,
            42 => FS3,
            43 => G3,
            44 => GS3,
            45 => A3,
            46 => AS3,
            47 => B3,
            48 => C4,
            49 => CS4,
            50 => D4,
            51 => DS4,
            52 => E4,
            53 => F4,
            54 => FS4,
            55 => G4,
            56 => GS4,
            57 => A4,
            58 => AS4,
            59 => B4,
            60 => C5,
            61 => CS5,
            62 => D5,
            63 => DS5,
            64 => E5,
            65 => F5,
            66 => FS5,
            67 => G5,
            68 => GS5,
            69 => A5,
            70 => AS5,
            71 => B5,
            72 => C6,
            73 => CS6,
            74 => D6,
            75 => DS6,
            76 => E6,
            77 => F6,
            78 => FS6,
            79 => G6,
            80 => GS6,
            81 => A6,
            82 => AS6,
            83 => B6,
            84 => C7,
            85 => CS7,
            86 => D7,
            87 => DS7,
            88 => E7,
            89 => F7,
            90 => FS7,
            91 => G7,
            92 => GS7,
            93 => A7,
            94 => AS7,
            95 => B7,
            96 => C8,
            97 => CS8,
            98 => D8,
            99 => DS8,
            100 => E8,
            101 => F8,
            102 => FS8,
            103 => G8,
            104 => GS8,
            105 => A8,
            106 => AS8,
            107 => B8,
            108 => C9,
            109 => CS9,
            110 => D9,
            111 => DS9,
            112 => E9,
            113 => F9,
            114 => FS9,
            115 => G9,
            116 => GS9,
            117 => A9,
            118 => AS9,
            119 => B9,
            120 => C10,
            121 => CS10,
            122 => D10,
            123 => DS10,
            124 => E10,
            125 => F10,
            126 => FS10,
            127 => G10,
            128 => NoSound,
            n => bail!("Invalid number for note: {}", n),
        };

        Ok(note)
    }
}
