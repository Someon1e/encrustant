/// Magic key.
#[derive(Debug, Clone, Copy)]
pub struct Key {
    /// Multiplied with the bit board.
    pub magic: u64,

    /// Offset in the look up table index
    pub offset: u32,
}

/// Rook magic keys.
pub const ROOK_KEYS: [Key; 64] = [
    Key {
        magic: 0x0028_0077_ffeb_fffe,
        offset: 41305,
    },
    Key {
        magic: 0x2004_0102_0109_7fff,
        offset: 14326,
    },
    Key {
        magic: 0x0010_0200_1005_3fff,
        offset: 24477,
    },
    Key {
        magic: 0x0030_002f_f71f_fffa,
        offset: 8223,
    },
    Key {
        magic: 0x7fd0_0441_ffff_d003,
        offset: 49795,
    },
    Key {
        magic: 0x0040_01d9_e03f_fff7,
        offset: 60546,
    },
    Key {
        magic: 0x0040_0088_8847_ffff,
        offset: 28543,
    },
    Key {
        magic: 0x0068_00fb_ff75_fffd,
        offset: 79282,
    },
    Key {
        magic: 0x0000_2801_0113_ffff,
        offset: 6457,
    },
    Key {
        magic: 0x0020_0402_01fc_ffff,
        offset: 4125,
    },
    Key {
        magic: 0x007f_e800_42ff_ffe8,
        offset: 81021,
    },
    Key {
        magic: 0x0000_1800_217f_ffe8,
        offset: 42341,
    },
    Key {
        magic: 0x0000_1800_073f_ffe8,
        offset: 14139,
    },
    Key {
        magic: 0x007f_e800_9eff_ffe8,
        offset: 19465,
    },
    Key {
        magic: 0x0000_1800_602f_ffe8,
        offset: 9514,
    },
    Key {
        magic: 0x0000_3000_2fff_ffa0,
        offset: 71090,
    },
    Key {
        magic: 0x0030_0018_010b_ffff,
        offset: 75419,
    },
    Key {
        magic: 0x0003_000c_0085_fffb,
        offset: 33476,
    },
    Key {
        magic: 0x0004_0008_0201_0008,
        offset: 27117,
    },
    Key {
        magic: 0x0002_0020_0400_2002,
        offset: 85964,
    },
    Key {
        magic: 0x0002_0020_2001_0002,
        offset: 54915,
    },
    Key {
        magic: 0x0001_0020_2000_8001,
        offset: 36544,
    },
    Key {
        magic: 0x0000_0040_4000_8001,
        offset: 71854,
    },
    Key {
        magic: 0x0000_8020_0020_0040,
        offset: 37996,
    },
    Key {
        magic: 0x0040_2000_1008_0010,
        offset: 30398,
    },
    Key {
        magic: 0x0000_0800_1004_0010,
        offset: 55939,
    },
    Key {
        magic: 0x0004_0100_0802_0008,
        offset: 53891,
    },
    Key {
        magic: 0x0000_0400_2020_0200,
        offset: 56963,
    },
    Key {
        magic: 0x0000_0100_2002_0020,
        offset: 77451,
    },
    Key {
        magic: 0x0000_0100_2020_0080,
        offset: 12319,
    },
    Key {
        magic: 0x0000_0080_2020_0040,
        offset: 88500,
    },
    Key {
        magic: 0x0000_2000_2000_4081,
        offset: 51405,
    },
    Key {
        magic: 0x00ff_fd18_0030_0030,
        offset: 72878,
    },
    Key {
        magic: 0x007f_ff7f_bfd4_0020,
        offset: 676,
    },
    Key {
        magic: 0x003f_ffbd_0018_0018,
        offset: 83122,
    },
    Key {
        magic: 0x001f_ffde_8018_0018,
        offset: 22206,
    },
    Key {
        magic: 0x000f_ffe0_bfe8_0018,
        offset: 75186,
    },
    Key {
        magic: 0x0001_0000_8020_2001,
        offset: 681,
    },
    Key {
        magic: 0x0003_fffb_ff98_0180,
        offset: 36453,
    },
    Key {
        magic: 0x0001_fffd_ff90_00e0,
        offset: 20369,
    },
    Key {
        magic: 0x00ff_feeb_feff_d800,
        offset: 1981,
    },
    Key {
        magic: 0x007f_fff7_ffc0_1400,
        offset: 13343,
    },
    Key {
        magic: 0x0000_4081_0420_0204,
        offset: 10650,
    },
    Key {
        magic: 0x001f_fff0_1fc0_3000,
        offset: 57987,
    },
    Key {
        magic: 0x000f_ffe7_f8bf_e800,
        offset: 26302,
    },
    Key {
        magic: 0x0000_0080_0100_2020,
        offset: 58357,
    },
    Key {
        magic: 0x0003_fff8_5fff_a804,
        offset: 40546,
    },
    Key {
        magic: 0x0001_fffd_75ff_a802,
        offset: 0,
    },
    Key {
        magic: 0x00ff_ffec_0028_0028,
        offset: 14967,
    },
    Key {
        magic: 0x007f_ff75_ff7f_bfd8,
        offset: 80361,
    },
    Key {
        magic: 0x003f_ff86_3fbf_7fd8,
        offset: 40905,
    },
    Key {
        magic: 0x001f_ffbf_dfd7_ffd8,
        offset: 58347,
    },
    Key {
        magic: 0x000f_fff8_1028_0028,
        offset: 20381,
    },
    Key {
        magic: 0x0007_ffd7_f7fe_ffd8,
        offset: 81868,
    },
    Key {
        magic: 0x0003_fffc_0c48_0048,
        offset: 59381,
    },
    Key {
        magic: 0x0001_ffff_afd7_ffd8,
        offset: 84404,
    },
    Key {
        magic: 0x00ff_ffe4_ffdf_a3ba,
        offset: 45811,
    },
    Key {
        magic: 0x007f_ffef_7ff3_d3da,
        offset: 62898,
    },
    Key {
        magic: 0x003f_ffbf_dfef_f7fa,
        offset: 45796,
    },
    Key {
        magic: 0x001f_ffef_f7fb_fc22,
        offset: 66994,
    },
    Key {
        magic: 0x0000_0204_0800_1001,
        offset: 67204,
    },
    Key {
        magic: 0x0007_fffe_ffff_77fd,
        offset: 32448,
    },
    Key {
        magic: 0x0003_ffff_bf7d_feec,
        offset: 62946,
    },
    Key {
        magic: 0x0001_ffff_9dff_a333,
        offset: 17005,
    },
];

/// Bishop magic keys.
#[allow(clippy::unreadable_literal)]
pub const BISHOP_KEYS: [Key; 64] = [
    Key {
        magic: 0x0000_4040_4040_4040,
        offset: 33104,
    },
    Key {
        magic: 0x0000_a060_4010_07fc,
        offset: 4094,
    },
    Key {
        magic: 0x0000_4010_2020_0000,
        offset: 24764,
    },
    Key {
        magic: 0x0000_8060_0400_0000,
        offset: 13882,
    },
    Key {
        magic: 0x0000_4402_0000_0000,
        offset: 23090,
    },
    Key {
        magic: 0x0000_0801_0080_0000,
        offset: 32640,
    },
    Key {
        magic: 0x0000_1041_0400_4000,
        offset: 11558,
    },
    Key {
        magic: 0x0000_0200_2082_0080,
        offset: 32912,
    },
    Key {
        magic: 0x0000_0401_0020_2004,
        offset: 13674,
    },
    Key {
        magic: 0x0000_0200_8020_0802,
        offset: 6109,
    },
    Key {
        magic: 0x0000_0100_4008_0200,
        offset: 26494,
    },
    Key {
        magic: 0x0000_0080_6004_0000,
        offset: 17919,
    },
    Key {
        magic: 0x0000_0044_0200_0000,
        offset: 25757,
    },
    Key {
        magic: 0x0000_0021_c100_b200,
        offset: 17338,
    },
    Key {
        magic: 0x0000_0004_0041_0080,
        offset: 16983,
    },
    Key {
        magic: 0x0000_03f7_f05f_ffc0,
        offset: 16659,
    },
    Key {
        magic: 0x0004_2280_4080_8010,
        offset: 13610,
    },
    Key {
        magic: 0x0000_2000_4040_4040,
        offset: 2224,
    },
    Key {
        magic: 0x0000_4000_8080_8080,
        offset: 60405,
    },
    Key {
        magic: 0x0000_2002_0080_1000,
        offset: 7983,
    },
    Key {
        magic: 0x0000_2400_8084_0000,
        offset: 17,
    },
    Key {
        magic: 0x0000_1800_0c03_fff8,
        offset: 34321,
    },
    Key {
        magic: 0x0000_0a58_4020_8020,
        offset: 33216,
    },
    Key {
        magic: 0x0000_0584_0840_4010,
        offset: 17127,
    },
    Key {
        magic: 0x0002_0220_0040_8020,
        offset: 6397,
    },
    Key {
        magic: 0x0000_4020_0040_8080,
        offset: 22169,
    },
    Key {
        magic: 0x0000_8040_0081_0100,
        offset: 42727,
    },
    Key {
        magic: 0x0001_0040_3c04_03ff,
        offset: 155,
    },
    Key {
        magic: 0x0007_8402_a880_2000,
        offset: 8601,
    },
    Key {
        magic: 0x0000_1010_0080_4400,
        offset: 21101,
    },
    Key {
        magic: 0x0000_0808_0010_4100,
        offset: 29885,
    },
    Key {
        magic: 0x0000_4004_8010_1008,
        offset: 29340,
    },
    Key {
        magic: 0x0001_0101_0200_4040,
        offset: 19785,
    },
    Key {
        magic: 0x0000_8080_9040_2020,
        offset: 12258,
    },
    Key {
        magic: 0x0007_fefe_0881_0010,
        offset: 50451,
    },
    Key {
        magic: 0x0003_ff0f_833f_c080,
        offset: 1712,
    },
    Key {
        magic: 0x007f_e080_1900_3042,
        offset: 78475,
    },
    Key {
        magic: 0x0000_2020_4000_8040,
        offset: 7855,
    },
    Key {
        magic: 0x0001_0040_0838_1008,
        offset: 13642,
    },
    Key {
        magic: 0x0000_8020_0370_0808,
        offset: 8156,
    },
    Key {
        magic: 0x0000_2082_0040_0080,
        offset: 4348,
    },
    Key {
        magic: 0x0000_1041_0020_0040,
        offset: 28794,
    },
    Key {
        magic: 0x0003_ffdf_7f83_3fc0,
        offset: 22578,
    },
    Key {
        magic: 0x0000_0088_4045_0020,
        offset: 50315,
    },
    Key {
        magic: 0x0000_0200_4010_0100,
        offset: 85452,
    },
    Key {
        magic: 0x007f_ffdd_8014_0028,
        offset: 32816,
    },
    Key {
        magic: 0x0000_2020_2020_0040,
        offset: 13930,
    },
    Key {
        magic: 0x0001_0040_1003_9004,
        offset: 17967,
    },
    Key {
        magic: 0x0000_0400_4100_8000,
        offset: 33200,
    },
    Key {
        magic: 0x0003_ffef_e0c0_2200,
        offset: 32456,
    },
    Key {
        magic: 0x0000_0010_1080_6000,
        offset: 7762,
    },
    Key {
        magic: 0x0000_0000_0840_3000,
        offset: 7794,
    },
    Key {
        magic: 0x0000_0001_0020_2000,
        offset: 22761,
    },
    Key {
        magic: 0x0000_0401_0020_0800,
        offset: 14918,
    },
    Key {
        magic: 0x0000_4040_4040_4000,
        offset: 11620,
    },
    Key {
        magic: 0x0000_6020_6018_03f4,
        offset: 15925,
    },
    Key {
        magic: 0x0003_ffdf_dfc2_8048,
        offset: 32528,
    },
    Key {
        magic: 0x0000_0008_2082_0020,
        offset: 12196,
    },
    Key {
        magic: 0x0000_0000_1010_8060,
        offset: 32720,
    },
    Key {
        magic: 0x0000_0000_0008_4030,
        offset: 26781,
    },
    Key {
        magic: 0x0000_0000_0100_2020,
        offset: 19817,
    },
    Key {
        magic: 0x0000_0000_4040_8020,
        offset: 24732,
    },
    Key {
        magic: 0x0000_0040_4040_4040,
        offset: 25468,
    },
    Key {
        magic: 0x0000_4040_4040_4040,
        offset: 10186,
    },
];

/// Size of the move lookup table.
pub const SLIDERS_TABLE_SIZE: usize = 89524;

// Magic numbers taken from http://www.talkchess.com/forum/viewtopic.php?t=60065&start=14
