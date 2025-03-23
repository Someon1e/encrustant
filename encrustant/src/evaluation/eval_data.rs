//! Data used by evaluation.

/// Number type of the evaluation score.
pub type EvalNumber = i32;

/// Table containing the value of each piece for every square on the board.
pub type PieceSquareTable = [i16; 384];

/// Piece square tables for the middle game.
#[rustfmt::skip]
pub const MIDDLE_GAME_PIECE_SQUARE_TABLES: PieceSquareTable = [

  -2,  16,   2,  -4,   3,   2,   1,   4,
 157, 161, 148, 181, 164, 150,  73,  73,
  72,  88, 111, 119, 118, 137, 118,  81,
  47,  68,  70,  71,  89,  86,  93,  68,
  37,  62,  59,  74,  75,  72,  84,  56,
  38,  60,  61,  62,  75,  67,  96,  64,
  38,  62,  56,  43,  65,  80, 105,  61,
  13,   1,   3,   6,   3,   1,   0,   1,


 147, 188, 252, 266, 300, 232, 253, 204,
 260, 278, 318, 327, 314, 367, 289, 313,
 279, 313, 330, 342, 377, 370, 331, 311,
 286, 295, 313, 329, 313, 337, 305, 310,
 280, 286, 298, 299, 307, 306, 304, 286,
 265, 282, 290, 290, 299, 292, 297, 272,
 252, 261, 277, 284, 285, 287, 275, 276,
 222, 260, 252, 264, 267, 277, 257, 237,


 282, 265, 281, 241, 255, 254, 300, 288,
 288, 307, 310, 302, 323, 325, 312, 305,
 304, 330, 331, 351, 346, 368, 346, 333,
 304, 315, 334, 338, 335, 337, 319, 303,
 304, 314, 316, 330, 330, 320, 313, 309,
 316, 318, 317, 317, 319, 317, 320, 319,
 316, 317, 319, 307, 311, 322, 330, 316,
 300, 312, 302, 295, 298, 300, 315, 303,


 453, 456, 453, 470, 476, 495, 483, 510,
 445, 446, 467, 488, 478, 500, 487, 518,
 427, 442, 449, 454, 470, 470, 501, 485,
 414, 424, 431, 435, 438, 442, 453, 453,
 404, 409, 416, 423, 428, 421, 440, 432,
 403, 414, 416, 416, 423, 425, 453, 434,
 399, 408, 419, 419, 422, 425, 439, 416,
 419, 420, 427, 433, 436, 428, 442, 426,


 830, 837, 858, 893, 887, 921, 897, 892,
 860, 841, 854, 852, 848, 901, 885, 918,
 862, 868, 864, 876, 896, 921, 929, 923,
 858, 858, 860, 855, 860, 876, 878, 884,
 862, 860, 858, 862, 862, 865, 877, 878,
 859, 866, 862, 862, 866, 871, 881, 875,
 858, 862, 870, 870, 871, 877, 881, 885,
 864, 852, 857, 868, 862, 853, 854, 857,


 -35, -36, -34,-113, -75, -23,  15,  25,
 -50, -56, -92, -54, -61, -46,   3,  14,
 -90, -44, -82, -90, -65, -18, -20, -36,
-100, -89,-103,-131,-121, -92, -96, -86,
 -87, -93,-111,-144,-137,-116,-102,-115,
 -54, -52, -92,-109,-106, -92, -59, -67,
   3, -23, -41, -67, -72, -49, -13,  -6,
  -5,  19,   2, -80, -27, -56,   4,   4,

];

/// Piece square tables for the end game.
#[rustfmt::skip]
pub const END_GAME_PIECE_SQUARE_TABLES: PieceSquareTable = [

  72,  22,   7, -12,   8,   5,  12,  11,
 246, 245, 234, 190, 189, 189, 239, 242,
 201, 203, 180, 153, 147, 134, 173, 174,
 135, 126, 107,  99,  90,  91, 108, 112,
 108, 106,  90,  88,  83,  83,  94,  90,
 101, 105,  87,  98,  91,  86,  94,  84,
 104, 106,  94, 100, 104,  93,  92,  84,
  51,   9,   3,  -5,   0,   8,   1,  -1,


 230, 276, 278, 278, 275, 268, 261, 208,
 257, 279, 284, 285, 280, 270, 273, 248,
 273, 284, 297, 300, 285, 286, 278, 264,
 277, 297, 308, 311, 310, 308, 294, 275,
 279, 290, 307, 307, 308, 302, 290, 272,
 270, 283, 292, 304, 302, 291, 282, 269,
 264, 276, 281, 281, 281, 281, 272, 262,
 254, 242, 272, 271, 272, 265, 253, 247,


 295, 299, 297, 308, 302, 304, 291, 289,
 280, 300, 301, 299, 297, 294, 300, 276,
 299, 301, 308, 298, 299, 301, 298, 297,
 297, 311, 308, 321, 314, 310, 308, 296,
 294, 308, 317, 315, 314, 313, 307, 288,
 292, 308, 312, 311, 314, 311, 299, 290,
 295, 291, 296, 301, 302, 297, 297, 281,
 276, 292, 276, 292, 291, 290, 281, 276,


 519, 520, 528, 519, 518, 507, 509, 503,
 515, 522, 522, 515, 515, 504, 501, 489,
 515, 517, 516, 512, 505, 501, 493, 490,
 515, 514, 517, 514, 504, 502, 496, 492,
 510, 509, 513, 511, 504, 501, 489, 490,
 503, 504, 505, 506, 499, 493, 476, 480,
 500, 502, 505, 504, 497, 491, 482, 489,
 486, 497, 502, 502, 495, 488, 484, 474,


 994,1004,1011, 994, 996, 963, 971, 957,
 941, 985,1008,1023,1047, 985, 964, 936,
 944, 963, 999,1004,1002, 992, 948, 932,
 954, 978, 997,1018,1029,1012, 988, 964,
 954, 974, 991,1005,1005, 996, 969, 961,
 953, 958, 979, 978, 977, 970, 948, 940,
 944, 949, 946, 949, 951, 933, 907, 897,
 932, 943, 943, 936, 942, 935, 932, 929,


 -35,  -8,   1,  35,  22,  28,  18, -28,
   9,  34,  53,  44,  55,  63,  55,  25,
  23,  45,  56,  62,  67,  62,  64,  36,
  18,  46,  57,  67,  66,  64,  63,  36,
   9,  33,  48,  57,  59,  54,  44,  29,
  -2,  20,  34,  42,  43,  36,  25,  13,
 -17,   2,  15,  21,  25,  18,   7,  -8,
 -35, -24, -14,  -2, -21,  -3, -16, -41,

];

pub const PHASES: [i32; 5] = [-10, 88, 91, 186, 414];
