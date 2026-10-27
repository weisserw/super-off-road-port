//! Raw source resource boundaries, from ledger.json and the frozen Round29 source.
//! Physical ROM offsets only. Semantic decoding belongs to behavior contracts.
use super::Recipe;
pub const RECIPES: &[Recipe] = &[
    Recipe {
        id: "track_order_and_direction",
        offset: 0x216,
        length: 64,
    }, // region-372
    Recipe {
        id: "track_start_heading_00",
        offset: 0x68b,
        length: 1,
    }, // region-884
    Recipe {
        id: "start_heading_stride_storage_window",
        offset: 0x68c,
        length: 1,
    }, // region-1410
    Recipe {
        id: "track_start_heading_01",
        offset: 0x68d,
        length: 1,
    }, // region-885
    Recipe {
        id: "unresolved-data-region-1411",
        offset: 0x68e,
        length: 1,
    }, // region-1411
    Recipe {
        id: "track_start_heading_02",
        offset: 0x68f,
        length: 1,
    }, // region-886
    Recipe {
        id: "unresolved-data-region-1412",
        offset: 0x690,
        length: 1,
    }, // region-1412
    Recipe {
        id: "track_start_heading_03",
        offset: 0x691,
        length: 1,
    }, // region-887
    Recipe {
        id: "unresolved-data-region-1413",
        offset: 0x692,
        length: 1,
    }, // region-1413
    Recipe {
        id: "track_start_heading_04",
        offset: 0x693,
        length: 1,
    }, // region-888
    Recipe {
        id: "unresolved-data-region-1414",
        offset: 0x694,
        length: 1,
    }, // region-1414
    Recipe {
        id: "track_start_heading_05",
        offset: 0x695,
        length: 1,
    }, // region-889
    Recipe {
        id: "unresolved-data-region-1415",
        offset: 0x696,
        length: 1,
    }, // region-1415
    Recipe {
        id: "track_start_heading_06",
        offset: 0x697,
        length: 1,
    }, // region-890
    Recipe {
        id: "unresolved-data-region-1416",
        offset: 0x698,
        length: 1,
    }, // region-1416
    Recipe {
        id: "track_start_heading_07",
        offset: 0x699,
        length: 1,
    }, // region-891
    Recipe {
        id: "unresolved-data-region-1417",
        offset: 0x69a,
        length: 1,
    }, // region-1417
    Recipe {
        id: "track_start_heading_08",
        offset: 0x69b,
        length: 1,
    }, // region-892
    Recipe {
        id: "unresolved-data-region-1418",
        offset: 0x69c,
        length: 1,
    }, // region-1418
    Recipe {
        id: "track_start_heading_09",
        offset: 0x69d,
        length: 1,
    }, // region-893
    Recipe {
        id: "unresolved-data-region-1419",
        offset: 0x69e,
        length: 1,
    }, // region-1419
    Recipe {
        id: "track_start_heading_10",
        offset: 0x69f,
        length: 1,
    }, // region-894
    Recipe {
        id: "unresolved-data-region-1420",
        offset: 0x6a0,
        length: 1,
    }, // region-1420
    Recipe {
        id: "track_start_heading_11",
        offset: 0x6a1,
        length: 1,
    }, // region-895
    Recipe {
        id: "unresolved-data-region-1421",
        offset: 0x6a2,
        length: 1,
    }, // region-1421
    Recipe {
        id: "track_start_heading_12",
        offset: 0x6a3,
        length: 1,
    }, // region-896
    Recipe {
        id: "unresolved-data-region-1422",
        offset: 0x6a4,
        length: 1,
    }, // region-1422
    Recipe {
        id: "track_start_heading_13",
        offset: 0x6a5,
        length: 1,
    }, // region-897
    Recipe {
        id: "unresolved-data-region-1423",
        offset: 0x6a6,
        length: 1,
    }, // region-1423
    Recipe {
        id: "track_start_heading_14",
        offset: 0x6a7,
        length: 1,
    }, // region-898
    Recipe {
        id: "unresolved-data-region-1424",
        offset: 0x6a8,
        length: 1,
    }, // region-1424
    Recipe {
        id: "track_start_heading_15",
        offset: 0x6a9,
        length: 1,
    }, // region-899
    Recipe {
        id: "unresolved-data-region-1425",
        offset: 0x6aa,
        length: 1,
    }, // region-1425
    Recipe {
        id: "track_start_heading_16",
        offset: 0x6ab,
        length: 1,
    }, // region-900
    Recipe {
        id: "unresolved-data-region-1426",
        offset: 0x6ac,
        length: 1,
    }, // region-1426
    Recipe {
        id: "track_start_heading_17",
        offset: 0x6ad,
        length: 1,
    }, // region-901
    Recipe {
        id: "unresolved-data-region-1427",
        offset: 0x6ae,
        length: 1,
    }, // region-1427
    Recipe {
        id: "track_start_heading_18",
        offset: 0x6af,
        length: 1,
    }, // region-902
    Recipe {
        id: "unresolved-data-region-1428",
        offset: 0x6b0,
        length: 1,
    }, // region-1428
    Recipe {
        id: "track_start_heading_19",
        offset: 0x6b1,
        length: 1,
    }, // region-903
    Recipe {
        id: "unresolved-data-region-1429",
        offset: 0x6b2,
        length: 1,
    }, // region-1429
    Recipe {
        id: "track_start_heading_20",
        offset: 0x6b3,
        length: 1,
    }, // region-904
    Recipe {
        id: "unresolved-data-region-1430",
        offset: 0x6b4,
        length: 1,
    }, // region-1430
    Recipe {
        id: "track_start_heading_21",
        offset: 0x6b5,
        length: 1,
    }, // region-905
    Recipe {
        id: "unresolved-data-region-1431",
        offset: 0x6b6,
        length: 1,
    }, // region-1431
    Recipe {
        id: "track_start_heading_22",
        offset: 0x6b7,
        length: 1,
    }, // region-906
    Recipe {
        id: "unresolved-data-region-1432",
        offset: 0x6b8,
        length: 1,
    }, // region-1432
    Recipe {
        id: "track_start_heading_23",
        offset: 0x6b9,
        length: 1,
    }, // region-907
    Recipe {
        id: "unresolved-data-region-1433",
        offset: 0x6ba,
        length: 1,
    }, // region-1433
    Recipe {
        id: "track_start_heading_24",
        offset: 0x6bb,
        length: 1,
    }, // region-908
    Recipe {
        id: "unresolved-data-region-1434",
        offset: 0x6bc,
        length: 1,
    }, // region-1434
    Recipe {
        id: "track_start_heading_25",
        offset: 0x6bd,
        length: 1,
    }, // region-909
    Recipe {
        id: "unresolved-data-region-1435",
        offset: 0x6be,
        length: 1,
    }, // region-1435
    Recipe {
        id: "track_start_heading_26",
        offset: 0x6bf,
        length: 1,
    }, // region-910
    Recipe {
        id: "unresolved-data-region-1436",
        offset: 0x6c0,
        length: 1,
    }, // region-1436
    Recipe {
        id: "track_start_heading_27",
        offset: 0x6c1,
        length: 1,
    }, // region-911
    Recipe {
        id: "unresolved-data-region-1437",
        offset: 0x6c2,
        length: 1,
    }, // region-1437
    Recipe {
        id: "track_start_heading_28",
        offset: 0x6c3,
        length: 1,
    }, // region-912
    Recipe {
        id: "unresolved-data-region-1438",
        offset: 0x6c4,
        length: 1,
    }, // region-1438
    Recipe {
        id: "track_start_heading_29",
        offset: 0x6c5,
        length: 1,
    }, // region-913
    Recipe {
        id: "unresolved-data-region-1439",
        offset: 0x6c6,
        length: 1,
    }, // region-1439
    Recipe {
        id: "track_start_heading_30",
        offset: 0x6c7,
        length: 1,
    }, // region-914
    Recipe {
        id: "unresolved-data-region-1440",
        offset: 0x6c8,
        length: 1,
    }, // region-1440
    Recipe {
        id: "track_start_heading_31",
        offset: 0x6c9,
        length: 1,
    }, // region-915
    Recipe {
        id: "unresolved-data-region-1441",
        offset: 0x6ca,
        length: 1,
    }, // region-1441
    Recipe {
        id: "contact_animation_tile_stream",
        offset: 0x804,
        length: 24,
    }, // region-916
    Recipe {
        id: "unselected_contact_animation_tiles",
        offset: 0x81c,
        length: 7,
    }, // region-1369
    Recipe {
        id: "contact_animation_attribute_stream",
        offset: 0x823,
        length: 24,
    }, // region-917
    Recipe {
        id: "unselected_contact_animation_attributes",
        offset: 0x83b,
        length: 7,
    }, // region-1370
    Recipe {
        id: "ai_truck_turn_steps",
        offset: 0x989,
        length: 4,
    }, // region-918
    Recipe {
        id: "ai_parameter_waypoint_timeouts",
        offset: 0xa64,
        length: 8,
    }, // region-919
    Recipe {
        id: "unresolved-data-region-1442",
        offset: 0xa6c,
        length: 8,
    }, // region-1442
    Recipe {
        id: "truck_oam_high_bit_mask_0",
        offset: 0xaeb,
        length: 1,
    }, // region-920
    Recipe {
        id: "unresolved-data-region-1443",
        offset: 0xaec,
        length: 1,
    }, // region-1443
    Recipe {
        id: "truck_oam_high_bit_mask_1",
        offset: 0xaed,
        length: 1,
    }, // region-921
    Recipe {
        id: "unresolved-data-region-1444",
        offset: 0xaee,
        length: 1,
    }, // region-1444
    Recipe {
        id: "truck_oam_high_bit_mask_2",
        offset: 0xaef,
        length: 1,
    }, // region-922
    Recipe {
        id: "unresolved-data-region-1445",
        offset: 0xaf0,
        length: 1,
    }, // region-1445
    Recipe {
        id: "truck_oam_high_bit_mask_3",
        offset: 0xaf1,
        length: 1,
    }, // region-923
    Recipe {
        id: "unresolved-data-region-1446",
        offset: 0xaf2,
        length: 1,
    }, // region-1446
    Recipe {
        id: "tilt_pose_word_grid",
        offset: 0xca2,
        length: 50,
    }, // region-395
    Recipe {
        id: "blocked_coordinate_by_heading_quadrant",
        offset: 0xe7d,
        length: 4,
    }, // region-924
    Recipe {
        id: "track_countdown_oam_coordinates",
        offset: 0xf38,
        length: 32,
    }, // region-925
    Recipe {
        id: "countdown_effect_tiles",
        offset: 0xfe0,
        length: 12,
    }, // region-262
    Recipe {
        id: "race_countdown_tiles",
        offset: 0xfec,
        length: 70,
    }, // region-263
    Recipe {
        id: "track_parameter_arrays",
        offset: 0x123a,
        length: 240,
    }, // region-373
    Recipe {
        id: "track_graphics_source_pointers",
        offset: 0x13ef,
        length: 48,
    }, // region-374
    Recipe {
        id: "menu_direction_first_00",
        offset: 0x1438,
        length: 1,
    }, // region-927
    Recipe {
        id: "controller_direction_word_high_1439",
        offset: 0x1439,
        length: 1,
    }, // region-1282
    Recipe {
        id: "menu_direction_first_01",
        offset: 0x143a,
        length: 1,
    }, // region-928
    Recipe {
        id: "controller_direction_word_high_143b",
        offset: 0x143b,
        length: 1,
    }, // region-1283
    Recipe {
        id: "menu_direction_first_02",
        offset: 0x143c,
        length: 1,
    }, // region-929
    Recipe {
        id: "controller_direction_word_high_143d",
        offset: 0x143d,
        length: 1,
    }, // region-1284
    Recipe {
        id: "menu_direction_first_03",
        offset: 0x143e,
        length: 1,
    }, // region-930
    Recipe {
        id: "controller_direction_word_high_143f",
        offset: 0x143f,
        length: 1,
    }, // region-1285
    Recipe {
        id: "menu_direction_first_04",
        offset: 0x1440,
        length: 1,
    }, // region-931
    Recipe {
        id: "controller_direction_word_high_1441",
        offset: 0x1441,
        length: 1,
    }, // region-1286
    Recipe {
        id: "menu_direction_first_05",
        offset: 0x1442,
        length: 1,
    }, // region-932
    Recipe {
        id: "controller_direction_word_high_1443",
        offset: 0x1443,
        length: 1,
    }, // region-1287
    Recipe {
        id: "menu_direction_first_06",
        offset: 0x1444,
        length: 1,
    }, // region-933
    Recipe {
        id: "controller_direction_word_high_1445",
        offset: 0x1445,
        length: 1,
    }, // region-1288
    Recipe {
        id: "menu_direction_first_07",
        offset: 0x1446,
        length: 1,
    }, // region-934
    Recipe {
        id: "controller_direction_word_high_1447",
        offset: 0x1447,
        length: 1,
    }, // region-1289
    Recipe {
        id: "menu_direction_first_08",
        offset: 0x1448,
        length: 1,
    }, // region-935
    Recipe {
        id: "controller_direction_word_high_1449",
        offset: 0x1449,
        length: 1,
    }, // region-1290
    Recipe {
        id: "menu_direction_first_09",
        offset: 0x144a,
        length: 1,
    }, // region-936
    Recipe {
        id: "controller_direction_word_high_144b",
        offset: 0x144b,
        length: 1,
    }, // region-1291
    Recipe {
        id: "menu_direction_first_10",
        offset: 0x144c,
        length: 1,
    }, // region-937
    Recipe {
        id: "controller_direction_word_high_144d",
        offset: 0x144d,
        length: 1,
    }, // region-1292
    Recipe {
        id: "menu_direction_first_11",
        offset: 0x144e,
        length: 1,
    }, // region-938
    Recipe {
        id: "controller_direction_word_high_144f",
        offset: 0x144f,
        length: 1,
    }, // region-1293
    Recipe {
        id: "menu_direction_first_12",
        offset: 0x1450,
        length: 1,
    }, // region-939
    Recipe {
        id: "controller_direction_word_high_1451",
        offset: 0x1451,
        length: 1,
    }, // region-1294
    Recipe {
        id: "menu_direction_first_13",
        offset: 0x1452,
        length: 1,
    }, // region-940
    Recipe {
        id: "controller_direction_word_high_1453",
        offset: 0x1453,
        length: 1,
    }, // region-1295
    Recipe {
        id: "menu_direction_first_14",
        offset: 0x1454,
        length: 1,
    }, // region-941
    Recipe {
        id: "controller_direction_word_high_1455",
        offset: 0x1455,
        length: 1,
    }, // region-1296
    Recipe {
        id: "menu_direction_first_15",
        offset: 0x1456,
        length: 1,
    }, // region-942
    Recipe {
        id: "controller_direction_word_high_1457",
        offset: 0x1457,
        length: 1,
    }, // region-1297
    Recipe {
        id: "menu_direction_second_00",
        offset: 0x1458,
        length: 1,
    }, // region-943
    Recipe {
        id: "controller_direction_word_high_1459",
        offset: 0x1459,
        length: 1,
    }, // region-1298
    Recipe {
        id: "menu_direction_second_01",
        offset: 0x145a,
        length: 1,
    }, // region-944
    Recipe {
        id: "controller_direction_word_high_145b",
        offset: 0x145b,
        length: 1,
    }, // region-1299
    Recipe {
        id: "menu_direction_second_02",
        offset: 0x145c,
        length: 1,
    }, // region-945
    Recipe {
        id: "controller_direction_word_high_145d",
        offset: 0x145d,
        length: 1,
    }, // region-1300
    Recipe {
        id: "menu_direction_second_03",
        offset: 0x145e,
        length: 1,
    }, // region-946
    Recipe {
        id: "controller_direction_word_high_145f",
        offset: 0x145f,
        length: 1,
    }, // region-1301
    Recipe {
        id: "menu_direction_second_04",
        offset: 0x1460,
        length: 1,
    }, // region-947
    Recipe {
        id: "controller_direction_word_high_1461",
        offset: 0x1461,
        length: 1,
    }, // region-1302
    Recipe {
        id: "menu_direction_second_05",
        offset: 0x1462,
        length: 1,
    }, // region-948
    Recipe {
        id: "controller_direction_word_high_1463",
        offset: 0x1463,
        length: 1,
    }, // region-1303
    Recipe {
        id: "menu_direction_second_06",
        offset: 0x1464,
        length: 1,
    }, // region-949
    Recipe {
        id: "controller_direction_word_high_1465",
        offset: 0x1465,
        length: 1,
    }, // region-1304
    Recipe {
        id: "menu_direction_second_07",
        offset: 0x1466,
        length: 1,
    }, // region-950
    Recipe {
        id: "controller_direction_word_high_1467",
        offset: 0x1467,
        length: 1,
    }, // region-1305
    Recipe {
        id: "menu_direction_second_08",
        offset: 0x1468,
        length: 1,
    }, // region-951
    Recipe {
        id: "controller_direction_word_high_1469",
        offset: 0x1469,
        length: 1,
    }, // region-1306
    Recipe {
        id: "menu_direction_second_09",
        offset: 0x146a,
        length: 1,
    }, // region-952
    Recipe {
        id: "controller_direction_word_high_146b",
        offset: 0x146b,
        length: 1,
    }, // region-1307
    Recipe {
        id: "menu_direction_second_10",
        offset: 0x146c,
        length: 1,
    }, // region-953
    Recipe {
        id: "controller_direction_word_high_146d",
        offset: 0x146d,
        length: 1,
    }, // region-1308
    Recipe {
        id: "menu_direction_second_11",
        offset: 0x146e,
        length: 1,
    }, // region-954
    Recipe {
        id: "controller_direction_word_high_146f",
        offset: 0x146f,
        length: 1,
    }, // region-1309
    Recipe {
        id: "menu_direction_second_12",
        offset: 0x1470,
        length: 1,
    }, // region-955
    Recipe {
        id: "controller_direction_word_high_1471",
        offset: 0x1471,
        length: 1,
    }, // region-1310
    Recipe {
        id: "menu_direction_second_13",
        offset: 0x1472,
        length: 1,
    }, // region-956
    Recipe {
        id: "controller_direction_word_high_1473",
        offset: 0x1473,
        length: 1,
    }, // region-1311
    Recipe {
        id: "menu_direction_second_14",
        offset: 0x1474,
        length: 1,
    }, // region-957
    Recipe {
        id: "controller_direction_word_high_1475",
        offset: 0x1475,
        length: 1,
    }, // region-1312
    Recipe {
        id: "menu_direction_second_15",
        offset: 0x1476,
        length: 1,
    }, // region-958
    Recipe {
        id: "controller_direction_word_high_1477",
        offset: 0x1477,
        length: 1,
    }, // region-1313
    Recipe {
        id: "upgrade_speed_limits",
        offset: 0x1504,
        length: 9,
    }, // region-959
    Recipe {
        id: "upgrade_acceleration_fraction_steps",
        offset: 0x150d,
        length: 14,
    }, // region-960
    Recipe {
        id: "heading_x_vector_words",
        offset: 0x15dc,
        length: 64,
    }, // region-264
    Recipe {
        id: "heading_y_vector_words",
        offset: 0x161c,
        length: 64,
    }, // region-265
    Recipe {
        id: "upgrade_steering_rates",
        offset: 0x165c,
        length: 7,
    }, // region-961
    Recipe {
        id: "wheel_heading_phase_offsets",
        offset: 0x196a,
        length: 4,
    }, // region-962
    Recipe {
        id: "offset_grid_8x8",
        offset: 0x1afb,
        length: 128,
    }, // region-1409
    Recipe {
        id: "vector_heading_cells_00",
        offset: 0x1cc7,
        length: 8,
    }, // region-966
    Recipe {
        id: "vector_heading_x_minus8_y_0",
        offset: 0x1ccf,
        length: 1,
    }, // region-1373
    Recipe {
        id: "vector_heading_cells_09",
        offset: 0x1cd0,
        length: 15,
    }, // region-967
    Recipe {
        id: "vector_heading_x_minus8_y_1",
        offset: 0x1cdf,
        length: 1,
    }, // region-1374
    Recipe {
        id: "vector_heading_cells_19",
        offset: 0x1ce0,
        length: 15,
    }, // region-968
    Recipe {
        id: "vector_heading_x_minus8_y_2",
        offset: 0x1cef,
        length: 1,
    }, // region-1375
    Recipe {
        id: "vector_heading_cells_29",
        offset: 0x1cf0,
        length: 15,
    }, // region-969
    Recipe {
        id: "vector_heading_x_minus8_y_3",
        offset: 0x1cff,
        length: 1,
    }, // region-1376
    Recipe {
        id: "vector_heading_cells_39",
        offset: 0x1d00,
        length: 15,
    }, // region-970
    Recipe {
        id: "vector_heading_x_minus8_y_4",
        offset: 0x1d0f,
        length: 1,
    }, // region-1377
    Recipe {
        id: "vector_heading_cells_49",
        offset: 0x1d10,
        length: 15,
    }, // region-971
    Recipe {
        id: "vector_heading_x_minus8_y_5",
        offset: 0x1d1f,
        length: 1,
    }, // region-1378
    Recipe {
        id: "vector_heading_cells_59",
        offset: 0x1d20,
        length: 15,
    }, // region-972
    Recipe {
        id: "vector_heading_x_minus8_y_6",
        offset: 0x1d2f,
        length: 1,
    }, // region-1379
    Recipe {
        id: "vector_heading_cells_69",
        offset: 0x1d30,
        length: 15,
    }, // region-973
    Recipe {
        id: "vector_heading_x_minus8_y_7",
        offset: 0x1d3f,
        length: 1,
    }, // region-1380
    Recipe {
        id: "vector_heading_cells_79",
        offset: 0x1d40,
        length: 7,
    }, // region-974
    Recipe {
        id: "vector_heading_y_minus8_row",
        offset: 0x1d47,
        length: 16,
    }, // region-1381
    Recipe {
        id: "vector_heading_cells_90",
        offset: 0x1d57,
        length: 8,
    }, // region-975
    Recipe {
        id: "vector_heading_x_minus8_y_minus7",
        offset: 0x1d5f,
        length: 1,
    }, // region-1382
    Recipe {
        id: "vector_heading_cells_99",
        offset: 0x1d60,
        length: 15,
    }, // region-976
    Recipe {
        id: "vector_heading_x_minus8_y_minus6",
        offset: 0x1d6f,
        length: 1,
    }, // region-1383
    Recipe {
        id: "vector_heading_cells_a9",
        offset: 0x1d70,
        length: 15,
    }, // region-977
    Recipe {
        id: "vector_heading_x_minus8_y_minus5",
        offset: 0x1d7f,
        length: 1,
    }, // region-1384
    Recipe {
        id: "vector_heading_cells_b9",
        offset: 0x1d80,
        length: 15,
    }, // region-978
    Recipe {
        id: "vector_heading_x_minus8_y_minus4",
        offset: 0x1d8f,
        length: 1,
    }, // region-1385
    Recipe {
        id: "vector_heading_cells_c9",
        offset: 0x1d90,
        length: 15,
    }, // region-979
    Recipe {
        id: "vector_heading_x_minus8_y_minus3",
        offset: 0x1d9f,
        length: 1,
    }, // region-1386
    Recipe {
        id: "vector_heading_cells_d9",
        offset: 0x1da0,
        length: 15,
    }, // region-980
    Recipe {
        id: "vector_heading_x_minus8_y_minus2",
        offset: 0x1daf,
        length: 1,
    }, // region-1387
    Recipe {
        id: "vector_heading_cells_e9",
        offset: 0x1db0,
        length: 15,
    }, // region-981
    Recipe {
        id: "vector_heading_x_minus8_y_minus1",
        offset: 0x1dbf,
        length: 1,
    }, // region-1388
    Recipe {
        id: "vector_heading_cells_f9",
        offset: 0x1dc0,
        length: 7,
    }, // region-982
    Recipe {
        id: "heading_signed_sine",
        offset: 0x1dc7,
        length: 256,
    }, // region-983
    Recipe {
        id: "track_cgram_source_prefix",
        offset: 0x1ec7,
        length: 368,
    }, // region-984
    Recipe {
        id: "attract_track_order_slots",
        offset: 0x20de,
        length: 4,
    }, // region-806
    Recipe {
        id: "frontend_initialization_signature_bytes",
        offset: 0x21ae,
        length: 5,
    }, // region-985
    Recipe {
        id: "licensed_splash_rectangle_descriptor",
        offset: 0x21b3,
        length: 20,
    }, // region-65
    Recipe {
        id: "software_splash_rectangle_descriptor",
        offset: 0x21c7,
        length: 20,
    }, // region-67
    Recipe {
        id: "truck_participation_bits",
        offset: 0x2587,
        length: 4,
    }, // region-986
    Recipe {
        id: "ai_parameter_random_threshold_prefix",
        offset: 0x2687,
        length: 5,
    }, // region-987
    Recipe {
        id: "ai_and_human_rank_random_thresholds",
        offset: 0x268c,
        length: 3,
    }, // region-988
    Recipe {
        id: "human_rank_random_threshold_tail",
        offset: 0x268f,
        length: 5,
    }, // region-989
    Recipe {
        id: "paired_text_glyphs_2a32",
        offset: 0x2a32,
        length: 11,
    }, // region-990
    Recipe {
        id: "result_palette_offsets",
        offset: 0x2a3d,
        length: 6,
    }, // region-991
    Recipe {
        id: "paired_text_glyphs_2a43",
        offset: 0x2a43,
        length: 31,
    }, // region-992
    Recipe {
        id: "continue_picture_rectangle",
        offset: 0x2a62,
        length: 20,
    }, // region-314
    Recipe {
        id: "result_truck_participation_bits",
        offset: 0x2a76,
        length: 4,
    }, // region-993
    Recipe {
        id: "podium_cgram_destinations",
        offset: 0x2aa2,
        length: 3,
    }, // region-994
    Recipe {
        id: "podium_award_values",
        offset: 0x2c28,
        length: 3,
    }, // region-995
    Recipe {
        id: "unresolved-data-region-1447",
        offset: 0x2c2b,
        length: 1,
    }, // region-1447
    Recipe {
        id: "podium_vram_destination_pairs",
        offset: 0x2c2c,
        length: 12,
    }, // region-996
    Recipe {
        id: "shop_continue_cursor_tilewords",
        offset: 0x2da2,
        length: 18,
    }, // region-997
    Recipe {
        id: "text_glyphs_2db4",
        offset: 0x2db4,
        length: 60,
    }, // region-998
    Recipe {
        id: "cursor_coordinate_deltas",
        offset: 0x2f2a,
        length: 18,
    }, // region-999
    Recipe {
        id: "shop_item_costs_and_caps",
        offset: 0x2f3c,
        length: 10,
    }, // region-1000
    Recipe {
        id: "shop_upgrade_bar_destinations",
        offset: 0x310d,
        length: 20,
    }, // region-1001
    Recipe {
        id: "driver_heading_rectangle_descriptor",
        offset: 0x33c2,
        length: 20,
    }, // region-69
    Recipe {
        id: "continued_driver_rectangle",
        offset: 0x33d6,
        length: 20,
    }, // region-330
    Recipe {
        id: "driver1_panel_rectangle_descriptor",
        offset: 0x33ea,
        length: 20,
    }, // region-71
    Recipe {
        id: "alternate_player2_shop_rectangle",
        offset: 0x33fe,
        length: 20,
    }, // region-394
    Recipe {
        id: "driver2_panel_rectangle_descriptor",
        offset: 0x3412,
        length: 20,
    }, // region-73
    Recipe {
        id: "inline_driver_panel_heading_tiles",
        offset: 0x3463,
        length: 12,
    }, // region-1317
    Recipe {
        id: "window_hdma_descriptor_pair",
        offset: 0x3608,
        length: 14,
    }, // region-396
    Recipe {
        id: "window_shape_source_records",
        offset: 0x3616,
        length: 9,
    }, // region-397
    Recipe {
        id: "default_track_record_names",
        offset: 0x3688,
        length: 48,
    }, // region-59
    Recipe {
        id: "default_track_record_times",
        offset: 0x36b8,
        length: 32,
    }, // region-60
    Recipe {
        id: "default_truck_names",
        offset: 0x36d8,
        length: 12,
    }, // region-64
    Recipe {
        id: "truck_money_record_offsets",
        offset: 0x36e4,
        length: 4,
    }, // region-1002
    Recipe {
        id: "default_high_score_digits",
        offset: 0x3815,
        length: 60,
    }, // region-61
    Recipe {
        id: "default_high_score_names",
        offset: 0x3851,
        length: 30,
    }, // region-62
    Recipe {
        id: "default_high_score_order",
        offset: 0x386f,
        length: 10,
    }, // region-63
    Recipe {
        id: "highscore_heading_tiles",
        offset: 0x3a11,
        length: 10,
    }, // region-813
    Recipe {
        id: "highscore_score_row_destinations",
        offset: 0x3a1b,
        length: 20,
    }, // region-814
    Recipe {
        id: "highscore_name_row_destinations",
        offset: 0x3a2f,
        length: 20,
    }, // region-815
    Recipe {
        id: "paired_text_glyphs_3a43",
        offset: 0x3a43,
        length: 71,
    }, // region-1003
    Recipe {
        id: "text_glyphs_3a8a",
        offset: 0x3a8a,
        length: 22,
    }, // region-1004
    Recipe {
        id: "paired_text_glyphs_3aa0",
        offset: 0x3aa0,
        length: 12,
    }, // region-1005
    Recipe {
        id: "name_entry_text_records",
        offset: 0x3aac,
        length: 16,
    }, // region-1006
    Recipe {
        id: "temporary_name_defaults",
        offset: 0x3fe8,
        length: 10,
    }, // region-1008
    Recipe {
        id: "name_editor_cell_tables",
        offset: 0x3ff2,
        length: 105,
    }, // region-1009
    Recipe {
        id: "track_eight_overlay_descriptor",
        offset: 0x4123,
        length: 20,
    }, // region-398
    Recipe {
        id: "track_fourteen_overlay_descriptor",
        offset: 0x4137,
        length: 20,
    }, // region-399
    Recipe {
        id: "track_hud_layout_tables",
        offset: 0x4305,
        length: 384,
    }, // region-375
    Recipe {
        id: "pickup_spawn_records",
        offset: 0x46f3,
        length: 384,
    }, // region-400
    Recipe {
        id: "pickup_award_bases_and_tiles",
        offset: 0x48db,
        length: 20,
    }, // region-401
    Recipe {
        id: "unresolved-data-region-1448",
        offset: 0x48ef,
        length: 2,
    }, // region-1448
    Recipe {
        id: "pickup_state_tilewords",
        offset: 0x48f1,
        length: 20,
    }, // region-402
    Recipe {
        id: "dynamic_object_record_pointers",
        offset: 0x4984,
        length: 128,
    }, // region-403
    Recipe {
        id: "dynamic_object_group_00",
        offset: 0x4a04,
        length: 36,
    }, // region-404
    Recipe {
        id: "dynamic_object_group_01",
        offset: 0x4a28,
        length: 36,
    }, // region-405
    Recipe {
        id: "dynamic_object_group_02",
        offset: 0x4a4c,
        length: 36,
    }, // region-406
    Recipe {
        id: "dynamic_object_group_03",
        offset: 0x4a70,
        length: 36,
    }, // region-407
    Recipe {
        id: "dynamic_object_group_04",
        offset: 0x4a94,
        length: 36,
    }, // region-408
    Recipe {
        id: "dynamic_object_group_05",
        offset: 0x4ab8,
        length: 36,
    }, // region-409
    Recipe {
        id: "dynamic_object_group_06",
        offset: 0x4adc,
        length: 36,
    }, // region-410
    Recipe {
        id: "dynamic_object_group_07",
        offset: 0x4b00,
        length: 36,
    }, // region-411
    Recipe {
        id: "dynamic_object_group_08",
        offset: 0x4b24,
        length: 36,
    }, // region-412
    Recipe {
        id: "dynamic_object_group_09",
        offset: 0x4b48,
        length: 36,
    }, // region-413
    Recipe {
        id: "dynamic_object_group_10",
        offset: 0x4b6c,
        length: 36,
    }, // region-414
    Recipe {
        id: "dynamic_object_group_11",
        offset: 0x4b90,
        length: 36,
    }, // region-415
    Recipe {
        id: "dynamic_object_group_12",
        offset: 0x4bb4,
        length: 36,
    }, // region-416
    Recipe {
        id: "dynamic_object_group_13",
        offset: 0x4bd8,
        length: 36,
    }, // region-417
    Recipe {
        id: "dynamic_object_group_14",
        offset: 0x4bfc,
        length: 36,
    }, // region-418
    Recipe {
        id: "dynamic_object_group_15",
        offset: 0x4c20,
        length: 36,
    }, // region-419
    Recipe {
        id: "dynamic_object_group_16",
        offset: 0x4c44,
        length: 36,
    }, // region-420
    Recipe {
        id: "unresolved-data-region-1449",
        offset: 0x4dbe,
        length: 2,
    }, // region-1449
    Recipe {
        id: "dynamic_object_height_adjustments",
        offset: 0x4dc0,
        length: 6,
    }, // region-421
    Recipe {
        id: "dynamic_object_reaction_y_deltas",
        offset: 0x4dc6,
        length: 44,
    }, // region-422
    Recipe {
        id: "dynamic_object_animation_tiles",
        offset: 0x4df2,
        length: 70,
    }, // region-423
    Recipe {
        id: "track_three_sprite_coordinates",
        offset: 0x4ef9,
        length: 6,
    }, // region-424
    Recipe {
        id: "composite_object_tile_triplets",
        offset: 0x502e,
        length: 288,
    }, // region-425
    Recipe {
        id: "track_zero_composite_placement_pointer",
        offset: 0x514e,
        length: 2,
    }, // region-426
    Recipe {
        id: "unresolved-data-region-1450",
        offset: 0x5150,
        length: 14,
    }, // region-1450
    Recipe {
        id: "track_zero_composite_placements",
        offset: 0x515e,
        length: 26,
    }, // region-427
    Recipe {
        id: "placement_reaction_tilewords",
        offset: 0x5229,
        length: 112,
    }, // region-1010
    Recipe {
        id: "track_object_placement_bundle",
        offset: 0x5299,
        length: 400,
    }, // region-376
    Recipe {
        id: "countdown_track_window_x",
        offset: 0x55ee,
        length: 16,
    }, // region-822
    Recipe {
        id: "countdown_track_window_y",
        offset: 0x55fe,
        length: 16,
    }, // region-823
    Recipe {
        id: "countdown_track_vram_destinations",
        offset: 0x560e,
        length: 32,
    }, // region-824
    Recipe {
        id: "countdown_rectangle_sources",
        offset: 0x5667,
        length: 102,
    }, // region-825
    Recipe {
        id: "countdown_rectangle_descriptor",
        offset: 0x56cd,
        length: 20,
    }, // region-826
    Recipe {
        id: "pause_banner_descriptor",
        offset: 0x5780,
        length: 20,
    }, // region-428
    Recipe {
        id: "track_route_pointer_and_stream_bundle",
        offset: 0x57fa,
        length: 2052,
    }, // region-377
    Recipe {
        id: "snes_header_and_vectors",
        offset: 0x7fc0,
        length: 64,
    }, // region-51
    Recipe {
        id: "truck_pose_graphics_strips",
        offset: 0x8000,
        length: 53248,
    }, // region-429
    Recipe {
        id: "shared_sprites_and_track_asset_union",
        offset: 0x15000,
        length: 72704,
    }, // region-378
    Recipe {
        id: "track_terrain_union_04_to_07",
        offset: 0x28000,
        length: 23232,
    }, // region-379
    Recipe {
        id: "frontend_background_tiles_tail",
        offset: 0x2dac0,
        length: 2368,
    }, // region-54
    Recipe {
        id: "frontend_palette",
        offset: 0x2e400,
        length: 224,
    }, // region-55
    Recipe {
        id: "licensed_splash_tilemap",
        offset: 0x2e4e0,
        length: 900,
    }, // region-66
    Recipe {
        id: "software_splash_tilemap",
        offset: 0x2e864,
        length: 782,
    }, // region-68
    Recipe {
        id: "unresolved-data-region-1451",
        offset: 0x2eb72,
        length: 6,
    }, // region-1451
    Recipe {
        id: "track_background_map_union_00_to_06",
        offset: 0x30000,
        length: 21504,
    }, // region-380
    Recipe {
        id: "frontend_background_tiles",
        offset: 0x35400,
        length: 9600,
    }, // region-53
    Recipe {
        id: "podium_display_resource_bundle",
        offset: 0x38000,
        length: 16384,
    }, // region-295
    Recipe {
        id: "race_hud_graphics_union_prefix",
        offset: 0x3c000,
        length: 5632,
    }, // region-381
    Recipe {
        id: "countdown_tilemap_03d600",
        offset: 0x3d600,
        length: 12,
    }, // region-827
    Recipe {
        id: "countdown_tilemap_03d60c",
        offset: 0x3d60c,
        length: 98,
    }, // region-828
    Recipe {
        id: "countdown_tilemap_03d66e",
        offset: 0x3d66e,
        length: 98,
    }, // region-829
    Recipe {
        id: "countdown_tilemap_03d6d0",
        offset: 0x3d6d0,
        length: 98,
    }, // region-830
    Recipe {
        id: "countdown_tilemap_03d732",
        offset: 0x3d732,
        length: 98,
    }, // region-831
    Recipe {
        id: "countdown_tilemap_03d794",
        offset: 0x3d794,
        length: 98,
    }, // region-832
    Recipe {
        id: "countdown_tilemap_03d7f6",
        offset: 0x3d7f6,
        length: 98,
    }, // region-833
    Recipe {
        id: "countdown_tilemap_03d858",
        offset: 0x3d858,
        length: 98,
    }, // region-834
    Recipe {
        id: "countdown_tilemap_03d8ba",
        offset: 0x3d8ba,
        length: 98,
    }, // region-835
    Recipe {
        id: "countdown_tilemap_03d91c",
        offset: 0x3d91c,
        length: 98,
    }, // region-836
    Recipe {
        id: "track_title_map_union",
        offset: 0x3d97e,
        length: 310,
    }, // region-382
    Recipe {
        id: "pause_banner_tile_rectangle",
        offset: 0x3dab4,
        length: 30,
    }, // region-430
    Recipe {
        id: "post_race_vram_transfer",
        offset: 0x3dad2,
        length: 3520,
    }, // region-1011
    Recipe {
        id: "countdown_tilemap_03e892",
        offset: 0x3e892,
        length: 98,
    }, // region-837
    Recipe {
        id: "countdown_tilemap_03e8f4",
        offset: 0x3e8f4,
        length: 98,
    }, // region-838
    Recipe {
        id: "countdown_tilemap_03e956",
        offset: 0x3e956,
        length: 98,
    }, // region-839
    Recipe {
        id: "countdown_tilemap_03e9b8",
        offset: 0x3e9b8,
        length: 98,
    }, // region-840
    Recipe {
        id: "countdown_tilemap_03ea1a",
        offset: 0x3ea1a,
        length: 98,
    }, // region-841
    Recipe {
        id: "countdown_tilemap_03ea7c",
        offset: 0x3ea7c,
        length: 98,
    }, // region-842
    Recipe {
        id: "countdown_tilemap_03eade",
        offset: 0x3eade,
        length: 98,
    }, // region-843
    Recipe {
        id: "countdown_tilemap_03eb40",
        offset: 0x3eb40,
        length: 98,
    }, // region-844
    Recipe {
        id: "countdown_tilemap_03eba2",
        offset: 0x3eba2,
        length: 98,
    }, // region-845
    Recipe {
        id: "countdown_tilemap_03ec04",
        offset: 0x3ec04,
        length: 98,
    }, // region-846
    Recipe {
        id: "countdown_tilemap_03ec66",
        offset: 0x3ec66,
        length: 98,
    }, // region-847
    Recipe {
        id: "countdown_tilemap_03ecc8",
        offset: 0x3ecc8,
        length: 98,
    }, // region-848
    Recipe {
        id: "countdown_tilemap_03ed2a",
        offset: 0x3ed2a,
        length: 98,
    }, // region-849
    Recipe {
        id: "countdown_tilemap_03ed8c",
        offset: 0x3ed8c,
        length: 98,
    }, // region-850
    Recipe {
        id: "countdown_tilemap_03edee",
        offset: 0x3edee,
        length: 98,
    }, // region-851
    Recipe {
        id: "countdown_tilemap_03ee50",
        offset: 0x3ee50,
        length: 98,
    }, // region-852
    Recipe {
        id: "attract_display_graphics",
        offset: 0x40000,
        length: 19712,
    }, // region-853
    Recipe {
        id: "attract_display_tilemap",
        offset: 0x44d00,
        length: 1536,
    }, // region-854
    Recipe {
        id: "unresolved-data-region-1452",
        offset: 0x45300,
        length: 64,
    }, // region-1452
    Recipe {
        id: "attract_display_palette",
        offset: 0x45340,
        length: 256,
    }, // region-855
    Recipe {
        id: "frontend_font_sprite_palette_union",
        offset: 0x45440,
        length: 3584,
    }, // region-52
    Recipe {
        id: "track_banner_map",
        offset: 0x46240,
        length: 32,
    }, // region-383
    Recipe {
        id: "unresolved-data-region-1453",
        offset: 0x46260,
        length: 58,
    }, // region-1453
    Recipe {
        id: "window_shape_samples_0",
        offset: 0x4629a,
        length: 256,
    }, // region-431
    Recipe {
        id: "window_shape_samples_1",
        offset: 0x4639a,
        length: 256,
    }, // region-432
    Recipe {
        id: "window_shape_samples_2",
        offset: 0x4649a,
        length: 256,
    }, // region-433
    Recipe {
        id: "continue_picture_palette",
        offset: 0x4659a,
        length: 192,
    }, // region-313
    Recipe {
        id: "continue_picture_tilemap",
        offset: 0x4665a,
        length: 1472,
    }, // region-315
    Recipe {
        id: "result_palette_replacements",
        offset: 0x46c1a,
        length: 288,
    }, // region-1012
    Recipe {
        id: "driver_frontend_tiles",
        offset: 0x48000,
        length: 19872,
    }, // region-56
    Recipe {
        id: "driver_heading_tilemap",
        offset: 0x4cda0,
        length: 448,
    }, // region-70
    Recipe {
        id: "driver1_panel_tilemap",
        offset: 0x4cf60,
        length: 504,
    }, // region-72
    Recipe {
        id: "alternate_driver_palette",
        offset: 0x4d158,
        length: 64,
    }, // region-58
    Recipe {
        id: "continued_driver_tilemap",
        offset: 0x4d198,
        length: 504,
    }, // region-331
    Recipe {
        id: "continued_driver_palette",
        offset: 0x4d390,
        length: 32,
    }, // region-329
    Recipe {
        id: "second_page_palette_zero",
        offset: 0x4d3b0,
        length: 32,
    }, // region-434
    Recipe {
        id: "alternate_player2_shop_palette",
        offset: 0x4d3d0,
        length: 32,
    }, // region-393
    Recipe {
        id: "second_page_palette_one",
        offset: 0x4d3f0,
        length: 32,
    }, // region-435
    Recipe {
        id: "driver_frontend_palette",
        offset: 0x4d410,
        length: 256,
    }, // region-57
    Recipe {
        id: "continue_picture_tiles_a",
        offset: 0x4d510,
        length: 10880,
    }, // region-311
    Recipe {
        id: "track_terrain_union_08_to_11",
        offset: 0x50000,
        length: 23226,
    }, // region-384
    Recipe {
        id: "continue_picture_tiles_b",
        offset: 0x55aba,
        length: 4832,
    }, // region-312
    Recipe {
        id: "track_terrain_union_12_to_15",
        offset: 0x58000,
        length: 23233,
    }, // region-385
    Recipe {
        id: "track_graphics_and_background_union",
        offset: 0x60000,
        length: 35840,
    }, // region-386
    Recipe {
        id: "track_eight_overlay_tiles",
        offset: 0x68c00,
        length: 18,
    }, // region-436
    Recipe {
        id: "track_background_map_union_09_to_14",
        offset: 0x68c12,
        length: 18432,
    }, // region-387
    Recipe {
        id: "track_fourteen_overlay_one_tiles",
        offset: 0x6d412,
        length: 8,
    }, // region-437
    Recipe {
        id: "track_fourteen_overlay_two_tiles",
        offset: 0x6d41a,
        length: 8,
    }, // region-438
    Recipe {
        id: "track_background_map_union_07_and_15",
        offset: 0x6d422,
        length: 6144,
    }, // region-388
    Recipe {
        id: "spc_upload_block1_header",
        offset: 0x70000,
        length: 4,
    }, // region-88
    Recipe {
        id: "spc_envelope_dispatch_bytes",
        offset: 0x7048d,
        length: 10,
    }, // region-439
    Recipe {
        id: "spc_command_80_vector",
        offset: 0x70614,
        length: 2,
    }, // region-440
    Recipe {
        id: "spc_command_82_vector",
        offset: 0x70616,
        length: 2,
    }, // region-441
    Recipe {
        id: "spc_command_84_vector",
        offset: 0x70618,
        length: 2,
    }, // region-442
    Recipe {
        id: "spc_command_86_vector",
        offset: 0x7061a,
        length: 2,
    }, // region-443
    Recipe {
        id: "spc_command_88_vector",
        offset: 0x7061c,
        length: 2,
    }, // region-444
    Recipe {
        id: "spc_command_8a_vector",
        offset: 0x7061e,
        length: 2,
    }, // region-445
    Recipe {
        id: "spc_command_8c_vector",
        offset: 0x70620,
        length: 2,
    }, // region-446
    Recipe {
        id: "spc_command_8e_vector",
        offset: 0x70622,
        length: 2,
    }, // region-1348
    Recipe {
        id: "spc_command_90_vector",
        offset: 0x70624,
        length: 2,
    }, // region-447
    Recipe {
        id: "spc_command_92_vector",
        offset: 0x70626,
        length: 2,
    }, // region-448
    Recipe {
        id: "spc_command_94_vector",
        offset: 0x70628,
        length: 2,
    }, // region-449
    Recipe {
        id: "spc_command_96_vector",
        offset: 0x7062a,
        length: 2,
    }, // region-450
    Recipe {
        id: "spc_auxiliary_stream_command_vector",
        offset: 0x7062c,
        length: 2,
    }, // region-1013
    Recipe {
        id: "spc_command_9a_vector",
        offset: 0x7062e,
        length: 2,
    }, // region-451
    Recipe {
        id: "spc_command_9c_vector",
        offset: 0x70630,
        length: 2,
    }, // region-452
    Recipe {
        id: "spc_command_9e_vector",
        offset: 0x70632,
        length: 2,
    }, // region-453
    Recipe {
        id: "spc_command_a0_vector",
        offset: 0x70634,
        length: 2,
    }, // region-454
    Recipe {
        id: "spc_command_a2_vector",
        offset: 0x70636,
        length: 2,
    }, // region-455
    Recipe {
        id: "spc_command_a4_vector",
        offset: 0x70638,
        length: 2,
    }, // region-456
    Recipe {
        id: "spc_command_a6_vector",
        offset: 0x7063a,
        length: 2,
    }, // region-457
    Recipe {
        id: "spc_command_a8_vector",
        offset: 0x7063c,
        length: 2,
    }, // region-458
    Recipe {
        id: "spc_command_aa_vector",
        offset: 0x7063e,
        length: 2,
    }, // region-459
    Recipe {
        id: "spc_target_envelope_command_vector",
        offset: 0x70640,
        length: 2,
    }, // region-856
    Recipe {
        id: "spc_command_ae_vector",
        offset: 0x70642,
        length: 2,
    }, // region-460
    Recipe {
        id: "spc_command_b0_vector",
        offset: 0x70644,
        length: 2,
    }, // region-461
    Recipe {
        id: "spc_command_b2_vector",
        offset: 0x70646,
        length: 2,
    }, // region-462
    Recipe {
        id: "spc_command_b4_vector",
        offset: 0x70648,
        length: 2,
    }, // region-463
    Recipe {
        id: "spc_command_b6_vector",
        offset: 0x7064a,
        length: 2,
    }, // region-464
    Recipe {
        id: "spc_command_b8_vector",
        offset: 0x7064c,
        length: 2,
    }, // region-465
    Recipe {
        id: "spc_command_ba_vector",
        offset: 0x7064e,
        length: 2,
    }, // region-1349
    Recipe {
        id: "spc_command_bc_vector",
        offset: 0x70650,
        length: 2,
    }, // region-466
    Recipe {
        id: "spc_command_be_vector",
        offset: 0x70652,
        length: 2,
    }, // region-467
    Recipe {
        id: "spc_command_c0_vector",
        offset: 0x70654,
        length: 2,
    }, // region-468
    Recipe {
        id: "spc_command_c2_vector",
        offset: 0x70656,
        length: 2,
    }, // region-469
    Recipe {
        id: "spc_initial_dsp_registers",
        offset: 0x708b0,
        length: 26,
    }, // region-266
    Recipe {
        id: "spc_initial_dsp_values",
        offset: 0x708ca,
        length: 25,
    }, // region-267
    Recipe {
        id: "spc_voice_bit_complement_pairs",
        offset: 0x708e3,
        length: 15,
    }, // region-1014
    Recipe {
        id: "spc_pitch_selected_byte_000",
        offset: 0x708f2,
        length: 1,
    }, // region-1015
    Recipe {
        id: "spc_pitch_selected_byte_001",
        offset: 0x708f3,
        length: 1,
    }, // region-1016
    Recipe {
        id: "spc_pitch_selected_byte_002",
        offset: 0x708f4,
        length: 1,
    }, // region-1017
    Recipe {
        id: "spc_pitch_selected_byte_003",
        offset: 0x708f5,
        length: 1,
    }, // region-1018
    Recipe {
        id: "spc_pitch_selected_byte_004",
        offset: 0x708f6,
        length: 1,
    }, // region-1019
    Recipe {
        id: "spc_pitch_low_index_06_cell",
        offset: 0x708f7,
        length: 1,
    }, // region-1246
    Recipe {
        id: "spc_pitch_selected_byte_006",
        offset: 0x708f8,
        length: 1,
    }, // region-1020
    Recipe {
        id: "spc_pitch_low_index_08_cell",
        offset: 0x708f9,
        length: 1,
    }, // region-1247
    Recipe {
        id: "spc_pitch_selected_byte_008",
        offset: 0x708fa,
        length: 1,
    }, // region-1021
    Recipe {
        id: "spc_pitch_low_index_10_cell",
        offset: 0x708fb,
        length: 1,
    }, // region-1248
    Recipe {
        id: "spc_pitch_selected_byte_010",
        offset: 0x708fc,
        length: 1,
    }, // region-1022
    Recipe {
        id: "spc_pitch_selected_byte_011",
        offset: 0x708fd,
        length: 1,
    }, // region-1023
    Recipe {
        id: "r12_pitch_low_index_0d_0708fe",
        offset: 0x708fe,
        length: 1,
    }, // region-1389
    Recipe {
        id: "spc_pitch_low_index_14_cell",
        offset: 0x708ff,
        length: 1,
    }, // region-1249
    Recipe {
        id: "spc_pitch_selected_byte_014",
        offset: 0x70900,
        length: 1,
    }, // region-1024
    Recipe {
        id: "spc_pitch_low_index_16_cell",
        offset: 0x70901,
        length: 1,
    }, // region-1250
    Recipe {
        id: "r12_pitch_low_index_11_070902",
        offset: 0x70902,
        length: 1,
    }, // region-1391
    Recipe {
        id: "spc_pitch_low_index_18_cell",
        offset: 0x70903,
        length: 1,
    }, // region-1251
    Recipe {
        id: "r12_pitch_low_index_13_070904",
        offset: 0x70904,
        length: 1,
    }, // region-1393
    Recipe {
        id: "spc_pitch_low_index_20_cell",
        offset: 0x70905,
        length: 1,
    }, // region-1252
    Recipe {
        id: "r12_pitch_low_index_15_070906",
        offset: 0x70906,
        length: 1,
    }, // region-1395
    Recipe {
        id: "spc_pitch_low_index_22_cell",
        offset: 0x70907,
        length: 1,
    }, // region-1253
    Recipe {
        id: "r12_pitch_low_index_17_070908",
        offset: 0x70908,
        length: 1,
    }, // region-1397
    Recipe {
        id: "spc_pitch_selected_byte_023",
        offset: 0x70909,
        length: 1,
    }, // region-1025
    Recipe {
        id: "spc_pitch_low_index_25_cell",
        offset: 0x7090a,
        length: 1,
    }, // region-1254
    Recipe {
        id: "spc_pitch_selected_byte_025",
        offset: 0x7090b,
        length: 1,
    }, // region-1026
    Recipe {
        id: "r12_pitch_low_index_1b_07090c",
        offset: 0x7090c,
        length: 1,
    }, // region-1399
    Recipe {
        id: "spc_pitch_selected_byte_027",
        offset: 0x7090d,
        length: 1,
    }, // region-1027
    Recipe {
        id: "spc_pitch_selected_byte_028",
        offset: 0x7090e,
        length: 1,
    }, // region-1028
    Recipe {
        id: "spc_pitch_low_index_30_cell",
        offset: 0x7090f,
        length: 1,
    }, // region-1255
    Recipe {
        id: "spc_pitch_selected_byte_030",
        offset: 0x70910,
        length: 1,
    }, // region-1029
    Recipe {
        id: "spc_pitch_selected_byte_031",
        offset: 0x70911,
        length: 1,
    }, // region-1030
    Recipe {
        id: "spc_pitch_selected_byte_032",
        offset: 0x70912,
        length: 1,
    }, // region-1031
    Recipe {
        id: "spc_pitch_selected_byte_033",
        offset: 0x70913,
        length: 1,
    }, // region-1032
    Recipe {
        id: "spc_pitch_selected_byte_034",
        offset: 0x70914,
        length: 1,
    }, // region-1033
    Recipe {
        id: "spc_pitch_selected_byte_035",
        offset: 0x70915,
        length: 1,
    }, // region-1034
    Recipe {
        id: "spc_pitch_selected_byte_036",
        offset: 0x70916,
        length: 1,
    }, // region-1035
    Recipe {
        id: "spc_pitch_selected_byte_037",
        offset: 0x70917,
        length: 1,
    }, // region-1036
    Recipe {
        id: "spc_pitch_selected_byte_038",
        offset: 0x70918,
        length: 1,
    }, // region-1037
    Recipe {
        id: "spc_pitch_selected_byte_039",
        offset: 0x70919,
        length: 1,
    }, // region-1038
    Recipe {
        id: "spc_pitch_selected_byte_040",
        offset: 0x7091a,
        length: 1,
    }, // region-1039
    Recipe {
        id: "spc_pitch_selected_byte_041",
        offset: 0x7091b,
        length: 1,
    }, // region-1040
    Recipe {
        id: "spc_pitch_selected_byte_042",
        offset: 0x7091c,
        length: 1,
    }, // region-1041
    Recipe {
        id: "spc_pitch_selected_byte_043",
        offset: 0x7091d,
        length: 1,
    }, // region-1042
    Recipe {
        id: "spc_pitch_selected_byte_044",
        offset: 0x7091e,
        length: 1,
    }, // region-1043
    Recipe {
        id: "spc_pitch_selected_byte_045",
        offset: 0x7091f,
        length: 1,
    }, // region-1044
    Recipe {
        id: "spc_pitch_selected_byte_046",
        offset: 0x70920,
        length: 1,
    }, // region-1045
    Recipe {
        id: "spc_pitch_selected_byte_047",
        offset: 0x70921,
        length: 1,
    }, // region-1046
    Recipe {
        id: "spc_pitch_selected_byte_048",
        offset: 0x70922,
        length: 1,
    }, // region-1047
    Recipe {
        id: "spc_pitch_selected_byte_049",
        offset: 0x70923,
        length: 1,
    }, // region-1048
    Recipe {
        id: "spc_pitch_selected_byte_050",
        offset: 0x70924,
        length: 1,
    }, // region-1049
    Recipe {
        id: "spc_pitch_selected_byte_051",
        offset: 0x70925,
        length: 1,
    }, // region-1050
    Recipe {
        id: "spc_pitch_selected_byte_052",
        offset: 0x70926,
        length: 1,
    }, // region-1051
    Recipe {
        id: "spc_pitch_selected_byte_053",
        offset: 0x70927,
        length: 1,
    }, // region-1052
    Recipe {
        id: "spc_pitch_selected_byte_054",
        offset: 0x70928,
        length: 1,
    }, // region-1053
    Recipe {
        id: "spc_pitch_selected_byte_055",
        offset: 0x70929,
        length: 1,
    }, // region-1054
    Recipe {
        id: "spc_pitch_selected_byte_056",
        offset: 0x7092a,
        length: 1,
    }, // region-1055
    Recipe {
        id: "spc_pitch_selected_byte_057",
        offset: 0x7092b,
        length: 1,
    }, // region-1056
    Recipe {
        id: "spc_pitch_selected_byte_058",
        offset: 0x7092c,
        length: 1,
    }, // region-1057
    Recipe {
        id: "spc_pitch_selected_byte_059",
        offset: 0x7092d,
        length: 1,
    }, // region-1058
    Recipe {
        id: "spc_pitch_selected_byte_060",
        offset: 0x7092e,
        length: 1,
    }, // region-1059
    Recipe {
        id: "spc_pitch_selected_byte_061",
        offset: 0x7092f,
        length: 1,
    }, // region-1060
    Recipe {
        id: "spc_pitch_selected_byte_062",
        offset: 0x70930,
        length: 1,
    }, // region-1061
    Recipe {
        id: "spc_pitch_selected_byte_063",
        offset: 0x70931,
        length: 1,
    }, // region-1062
    Recipe {
        id: "spc_pitch_selected_byte_064",
        offset: 0x70932,
        length: 1,
    }, // region-1063
    Recipe {
        id: "spc_pitch_selected_byte_065",
        offset: 0x70933,
        length: 1,
    }, // region-1064
    Recipe {
        id: "spc_pitch_selected_byte_066",
        offset: 0x70934,
        length: 1,
    }, // region-1065
    Recipe {
        id: "spc_pitch_selected_byte_067",
        offset: 0x70935,
        length: 1,
    }, // region-1066
    Recipe {
        id: "spc_pitch_selected_byte_068",
        offset: 0x70936,
        length: 1,
    }, // region-1067
    Recipe {
        id: "spc_pitch_selected_byte_069",
        offset: 0x70937,
        length: 1,
    }, // region-1068
    Recipe {
        id: "spc_pitch_selected_byte_070",
        offset: 0x70938,
        length: 1,
    }, // region-1069
    Recipe {
        id: "spc_pitch_selected_byte_071",
        offset: 0x70939,
        length: 1,
    }, // region-1070
    Recipe {
        id: "spc_pitch_selected_byte_072",
        offset: 0x7093a,
        length: 1,
    }, // region-1071
    Recipe {
        id: "spc_pitch_selected_byte_073",
        offset: 0x7093b,
        length: 1,
    }, // region-1072
    Recipe {
        id: "spc_pitch_selected_byte_074",
        offset: 0x7093c,
        length: 1,
    }, // region-1073
    Recipe {
        id: "spc_pitch_selected_byte_075",
        offset: 0x7093d,
        length: 1,
    }, // region-1074
    Recipe {
        id: "spc_pitch_selected_byte_076",
        offset: 0x7093e,
        length: 1,
    }, // region-1075
    Recipe {
        id: "spc_pitch_selected_byte_077",
        offset: 0x7093f,
        length: 1,
    }, // region-1076
    Recipe {
        id: "spc_pitch_selected_byte_078",
        offset: 0x70940,
        length: 1,
    }, // region-1077
    Recipe {
        id: "spc_pitch_selected_byte_079",
        offset: 0x70941,
        length: 1,
    }, // region-1078
    Recipe {
        id: "spc_pitch_selected_byte_080",
        offset: 0x70942,
        length: 1,
    }, // region-1079
    Recipe {
        id: "r12_pitch_low_index_52_070943",
        offset: 0x70943,
        length: 1,
    }, // region-1401
    Recipe {
        id: "r12_pitch_low_index_53_070944",
        offset: 0x70944,
        length: 1,
    }, // region-1403
    Recipe {
        id: "r12_pitch_low_index_54_070945",
        offset: 0x70945,
        length: 1,
    }, // region-1405
    Recipe {
        id: "spc_pitch_high_index_00_cell",
        offset: 0x70946,
        length: 1,
    }, // region-1256
    Recipe {
        id: "spc_pitch_selected_byte_085",
        offset: 0x70947,
        length: 1,
    }, // region-1080
    Recipe {
        id: "spc_pitch_selected_byte_086",
        offset: 0x70948,
        length: 1,
    }, // region-1081
    Recipe {
        id: "spc_pitch_selected_byte_087",
        offset: 0x70949,
        length: 1,
    }, // region-1082
    Recipe {
        id: "spc_pitch_selected_byte_088",
        offset: 0x7094a,
        length: 1,
    }, // region-1083
    Recipe {
        id: "spc_pitch_selected_byte_089",
        offset: 0x7094b,
        length: 1,
    }, // region-1084
    Recipe {
        id: "spc_pitch_high_index_06_cell",
        offset: 0x7094c,
        length: 1,
    }, // region-1257
    Recipe {
        id: "spc_pitch_selected_byte_091",
        offset: 0x7094d,
        length: 1,
    }, // region-1085
    Recipe {
        id: "spc_pitch_high_index_08_cell",
        offset: 0x7094e,
        length: 1,
    }, // region-1258
    Recipe {
        id: "spc_pitch_selected_byte_093",
        offset: 0x7094f,
        length: 1,
    }, // region-1086
    Recipe {
        id: "spc_pitch_high_index_10_cell",
        offset: 0x70950,
        length: 1,
    }, // region-1259
    Recipe {
        id: "spc_pitch_selected_byte_095",
        offset: 0x70951,
        length: 1,
    }, // region-1087
    Recipe {
        id: "spc_pitch_selected_byte_096",
        offset: 0x70952,
        length: 1,
    }, // region-1088
    Recipe {
        id: "r12_pitch_high_index_0d_070953",
        offset: 0x70953,
        length: 1,
    }, // region-1390
    Recipe {
        id: "spc_pitch_high_index_14_cell",
        offset: 0x70954,
        length: 1,
    }, // region-1260
    Recipe {
        id: "spc_pitch_selected_byte_099",
        offset: 0x70955,
        length: 1,
    }, // region-1089
    Recipe {
        id: "spc_pitch_high_index_16_cell",
        offset: 0x70956,
        length: 1,
    }, // region-1261
    Recipe {
        id: "r12_pitch_high_index_11_070957",
        offset: 0x70957,
        length: 1,
    }, // region-1392
    Recipe {
        id: "spc_pitch_high_index_18_cell",
        offset: 0x70958,
        length: 1,
    }, // region-1262
    Recipe {
        id: "r12_pitch_high_index_13_070959",
        offset: 0x70959,
        length: 1,
    }, // region-1394
    Recipe {
        id: "spc_pitch_high_index_20_cell",
        offset: 0x7095a,
        length: 1,
    }, // region-1263
    Recipe {
        id: "r12_pitch_high_index_15_07095b",
        offset: 0x7095b,
        length: 1,
    }, // region-1396
    Recipe {
        id: "spc_pitch_high_index_22_cell",
        offset: 0x7095c,
        length: 1,
    }, // region-1264
    Recipe {
        id: "r12_pitch_high_index_17_07095d",
        offset: 0x7095d,
        length: 1,
    }, // region-1398
    Recipe {
        id: "spc_pitch_selected_byte_108",
        offset: 0x7095e,
        length: 1,
    }, // region-1090
    Recipe {
        id: "spc_pitch_high_index_25_cell",
        offset: 0x7095f,
        length: 1,
    }, // region-1265
    Recipe {
        id: "spc_pitch_selected_byte_110",
        offset: 0x70960,
        length: 1,
    }, // region-1091
    Recipe {
        id: "r12_pitch_high_index_1b_070961",
        offset: 0x70961,
        length: 1,
    }, // region-1400
    Recipe {
        id: "spc_pitch_selected_byte_112",
        offset: 0x70962,
        length: 1,
    }, // region-1092
    Recipe {
        id: "spc_pitch_selected_byte_113",
        offset: 0x70963,
        length: 1,
    }, // region-1093
    Recipe {
        id: "spc_pitch_high_index_30_cell",
        offset: 0x70964,
        length: 1,
    }, // region-1266
    Recipe {
        id: "spc_pitch_selected_byte_115",
        offset: 0x70965,
        length: 1,
    }, // region-1094
    Recipe {
        id: "spc_pitch_selected_byte_116",
        offset: 0x70966,
        length: 1,
    }, // region-1095
    Recipe {
        id: "spc_pitch_selected_byte_117",
        offset: 0x70967,
        length: 1,
    }, // region-1096
    Recipe {
        id: "spc_pitch_selected_byte_118",
        offset: 0x70968,
        length: 1,
    }, // region-1097
    Recipe {
        id: "spc_pitch_selected_byte_119",
        offset: 0x70969,
        length: 1,
    }, // region-1098
    Recipe {
        id: "spc_pitch_selected_byte_120",
        offset: 0x7096a,
        length: 1,
    }, // region-1099
    Recipe {
        id: "spc_pitch_selected_byte_121",
        offset: 0x7096b,
        length: 1,
    }, // region-1100
    Recipe {
        id: "spc_pitch_selected_byte_122",
        offset: 0x7096c,
        length: 1,
    }, // region-1101
    Recipe {
        id: "spc_pitch_selected_byte_123",
        offset: 0x7096d,
        length: 1,
    }, // region-1102
    Recipe {
        id: "spc_pitch_selected_byte_124",
        offset: 0x7096e,
        length: 1,
    }, // region-1103
    Recipe {
        id: "spc_pitch_selected_byte_125",
        offset: 0x7096f,
        length: 1,
    }, // region-1104
    Recipe {
        id: "spc_pitch_selected_byte_126",
        offset: 0x70970,
        length: 1,
    }, // region-1105
    Recipe {
        id: "spc_pitch_selected_byte_127",
        offset: 0x70971,
        length: 1,
    }, // region-1106
    Recipe {
        id: "spc_pitch_selected_byte_128",
        offset: 0x70972,
        length: 1,
    }, // region-1107
    Recipe {
        id: "spc_pitch_selected_byte_129",
        offset: 0x70973,
        length: 1,
    }, // region-1108
    Recipe {
        id: "spc_pitch_selected_byte_130",
        offset: 0x70974,
        length: 1,
    }, // region-1109
    Recipe {
        id: "spc_pitch_selected_byte_131",
        offset: 0x70975,
        length: 1,
    }, // region-1110
    Recipe {
        id: "spc_pitch_selected_byte_132",
        offset: 0x70976,
        length: 1,
    }, // region-1111
    Recipe {
        id: "spc_pitch_selected_byte_133",
        offset: 0x70977,
        length: 1,
    }, // region-1112
    Recipe {
        id: "spc_pitch_selected_byte_134",
        offset: 0x70978,
        length: 1,
    }, // region-1113
    Recipe {
        id: "spc_pitch_selected_byte_135",
        offset: 0x70979,
        length: 1,
    }, // region-1114
    Recipe {
        id: "spc_pitch_selected_byte_136",
        offset: 0x7097a,
        length: 1,
    }, // region-1115
    Recipe {
        id: "spc_pitch_selected_byte_137",
        offset: 0x7097b,
        length: 1,
    }, // region-1116
    Recipe {
        id: "spc_pitch_selected_byte_138",
        offset: 0x7097c,
        length: 1,
    }, // region-1117
    Recipe {
        id: "spc_pitch_selected_byte_139",
        offset: 0x7097d,
        length: 1,
    }, // region-1118
    Recipe {
        id: "spc_pitch_selected_byte_140",
        offset: 0x7097e,
        length: 1,
    }, // region-1119
    Recipe {
        id: "spc_pitch_selected_byte_141",
        offset: 0x7097f,
        length: 1,
    }, // region-1120
    Recipe {
        id: "spc_pitch_selected_byte_142",
        offset: 0x70980,
        length: 1,
    }, // region-1121
    Recipe {
        id: "spc_pitch_selected_byte_143",
        offset: 0x70981,
        length: 1,
    }, // region-1122
    Recipe {
        id: "spc_pitch_selected_byte_144",
        offset: 0x70982,
        length: 1,
    }, // region-1123
    Recipe {
        id: "spc_pitch_selected_byte_145",
        offset: 0x70983,
        length: 1,
    }, // region-1124
    Recipe {
        id: "spc_pitch_selected_byte_146",
        offset: 0x70984,
        length: 1,
    }, // region-1125
    Recipe {
        id: "spc_pitch_selected_byte_147",
        offset: 0x70985,
        length: 1,
    }, // region-1126
    Recipe {
        id: "spc_pitch_selected_byte_148",
        offset: 0x70986,
        length: 1,
    }, // region-1127
    Recipe {
        id: "spc_pitch_selected_byte_149",
        offset: 0x70987,
        length: 1,
    }, // region-1128
    Recipe {
        id: "spc_pitch_selected_byte_150",
        offset: 0x70988,
        length: 1,
    }, // region-1129
    Recipe {
        id: "spc_pitch_selected_byte_151",
        offset: 0x70989,
        length: 1,
    }, // region-1130
    Recipe {
        id: "spc_pitch_selected_byte_152",
        offset: 0x7098a,
        length: 1,
    }, // region-1131
    Recipe {
        id: "spc_pitch_selected_byte_153",
        offset: 0x7098b,
        length: 1,
    }, // region-1132
    Recipe {
        id: "spc_pitch_selected_byte_154",
        offset: 0x7098c,
        length: 1,
    }, // region-1133
    Recipe {
        id: "spc_pitch_selected_byte_155",
        offset: 0x7098d,
        length: 1,
    }, // region-1134
    Recipe {
        id: "spc_pitch_selected_byte_156",
        offset: 0x7098e,
        length: 1,
    }, // region-1135
    Recipe {
        id: "spc_pitch_selected_byte_157",
        offset: 0x7098f,
        length: 1,
    }, // region-1136
    Recipe {
        id: "spc_pitch_selected_byte_158",
        offset: 0x70990,
        length: 1,
    }, // region-1137
    Recipe {
        id: "spc_pitch_selected_byte_159",
        offset: 0x70991,
        length: 1,
    }, // region-1138
    Recipe {
        id: "spc_pitch_selected_byte_160",
        offset: 0x70992,
        length: 1,
    }, // region-1139
    Recipe {
        id: "spc_pitch_selected_byte_161",
        offset: 0x70993,
        length: 1,
    }, // region-1140
    Recipe {
        id: "spc_pitch_selected_byte_162",
        offset: 0x70994,
        length: 1,
    }, // region-1141
    Recipe {
        id: "spc_pitch_selected_byte_163",
        offset: 0x70995,
        length: 1,
    }, // region-1142
    Recipe {
        id: "spc_pitch_selected_byte_164",
        offset: 0x70996,
        length: 1,
    }, // region-1143
    Recipe {
        id: "spc_pitch_selected_byte_165",
        offset: 0x70997,
        length: 1,
    }, // region-1144
    Recipe {
        id: "r12_pitch_high_index_52_070998",
        offset: 0x70998,
        length: 1,
    }, // region-1402
    Recipe {
        id: "r12_pitch_high_index_53_070999",
        offset: 0x70999,
        length: 1,
    }, // region-1404
    Recipe {
        id: "r12_pitch_high_index_54_07099a",
        offset: 0x7099a,
        length: 1,
    }, // region-1406
    Recipe {
        id: "r12_pitch_high_index_55_07099b",
        offset: 0x7099b,
        length: 1,
    }, // region-1407
    Recipe {
        id: "spc_conditional_effect_pitch_high_index86",
        offset: 0x7099c,
        length: 1,
    }, // region-1347
    Recipe {
        id: "unresolved-data-region-1454",
        offset: 0x7099d,
        length: 3,
    }, // region-1454
    Recipe {
        id: "spc_pitch_selected_byte_174",
        offset: 0x709a0,
        length: 1,
    }, // region-1145
    Recipe {
        id: "unresolved-data-region-1455",
        offset: 0x709a1,
        length: 73,
    }, // region-1455
    Recipe {
        id: "spc_pitch_selected_byte_248",
        offset: 0x709ea,
        length: 1,
    }, // region-1146
    Recipe {
        id: "unresolved-data-region-1456",
        offset: 0x709eb,
        length: 4,
    }, // region-1456
    Recipe {
        id: "spc_pitch_selected_byte_253",
        offset: 0x709ef,
        length: 1,
    }, // region-1147
    Recipe {
        id: "unresolved-data-region-1457",
        offset: 0x709f0,
        length: 52,
    }, // region-1457
    Recipe {
        id: "spc_song_00_voice_0_pointer_lo",
        offset: 0x70a24,
        length: 1,
    }, // region-470
    Recipe {
        id: "spc_song_01_voice_0_pointer_lo",
        offset: 0x70a25,
        length: 1,
    }, // region-471
    Recipe {
        id: "spc_song_02_voice_0_pointer_lo",
        offset: 0x70a26,
        length: 1,
    }, // region-472
    Recipe {
        id: "spc_song_03_voice_0_pointer_lo",
        offset: 0x70a27,
        length: 1,
    }, // region-473
    Recipe {
        id: "spc_song_04_voice_0_pointer_lo",
        offset: 0x70a28,
        length: 1,
    }, // region-474
    Recipe {
        id: "spc_song_05_12_voice_0_low",
        offset: 0x70a29,
        length: 8,
    }, // region-1148
    Recipe {
        id: "spc_song_13_voice_0_pointer_lo",
        offset: 0x70a31,
        length: 1,
    }, // region-475
    Recipe {
        id: "spc_song_14_voice_0_pointer_lo",
        offset: 0x70a32,
        length: 1,
    }, // region-476
    Recipe {
        id: "spc_song_15_voice_0_pointer_lo",
        offset: 0x70a33,
        length: 1,
    }, // region-477
    Recipe {
        id: "spc_reserved_voice0_low_slice_0e30",
        offset: 0x70a34,
        length: 11,
    }, // region-1350
    Recipe {
        id: "spc_pitch_selected_byte_333",
        offset: 0x70a3f,
        length: 1,
    }, // region-1149
    Recipe {
        id: "spc_reserved_voice0_low_slice_0e3c",
        offset: 0x70a40,
        length: 4,
    }, // region-1351
    Recipe {
        id: "spc_song_00_voice_0_pointer_hi",
        offset: 0x70a44,
        length: 1,
    }, // region-478
    Recipe {
        id: "spc_song_01_voice_0_pointer_hi",
        offset: 0x70a45,
        length: 1,
    }, // region-479
    Recipe {
        id: "spc_song_02_voice_0_pointer_hi",
        offset: 0x70a46,
        length: 1,
    }, // region-480
    Recipe {
        id: "spc_song_03_voice_0_pointer_hi",
        offset: 0x70a47,
        length: 1,
    }, // region-481
    Recipe {
        id: "spc_song_04_voice_0_pointer_hi",
        offset: 0x70a48,
        length: 1,
    }, // region-482
    Recipe {
        id: "spc_song_05_12_voice_0_high",
        offset: 0x70a49,
        length: 8,
    }, // region-1150
    Recipe {
        id: "spc_song_13_voice_0_pointer_hi",
        offset: 0x70a51,
        length: 1,
    }, // region-483
    Recipe {
        id: "spc_song_14_voice_0_pointer_hi",
        offset: 0x70a52,
        length: 1,
    }, // region-484
    Recipe {
        id: "spc_song_15_voice_0_pointer_hi",
        offset: 0x70a53,
        length: 1,
    }, // region-485
    Recipe {
        id: "spc_reserved_voice0_high_slice_0e50",
        offset: 0x70a54,
        length: 16,
    }, // region-1352
    Recipe {
        id: "spc_song_00_voice_1_pointer_lo",
        offset: 0x70a64,
        length: 1,
    }, // region-486
    Recipe {
        id: "spc_song_01_voice_1_pointer_lo",
        offset: 0x70a65,
        length: 1,
    }, // region-487
    Recipe {
        id: "spc_song_02_voice_1_pointer_lo",
        offset: 0x70a66,
        length: 1,
    }, // region-488
    Recipe {
        id: "spc_song_03_voice_1_pointer_lo",
        offset: 0x70a67,
        length: 1,
    }, // region-489
    Recipe {
        id: "spc_song_04_voice_1_pointer_lo",
        offset: 0x70a68,
        length: 1,
    }, // region-490
    Recipe {
        id: "spc_song_05_12_voice_1_low",
        offset: 0x70a69,
        length: 8,
    }, // region-1151
    Recipe {
        id: "spc_song_13_voice_1_pointer_lo",
        offset: 0x70a71,
        length: 1,
    }, // region-491
    Recipe {
        id: "spc_song_14_voice_1_pointer_lo",
        offset: 0x70a72,
        length: 1,
    }, // region-492
    Recipe {
        id: "spc_song_15_voice_1_pointer_lo",
        offset: 0x70a73,
        length: 1,
    }, // region-493
    Recipe {
        id: "spc_reserved_voice1_low_slice_0e70",
        offset: 0x70a74,
        length: 16,
    }, // region-1353
    Recipe {
        id: "spc_song_00_voice_1_pointer_hi",
        offset: 0x70a84,
        length: 1,
    }, // region-494
    Recipe {
        id: "spc_song_01_voice_1_pointer_hi",
        offset: 0x70a85,
        length: 1,
    }, // region-495
    Recipe {
        id: "spc_song_02_voice_1_pointer_hi",
        offset: 0x70a86,
        length: 1,
    }, // region-496
    Recipe {
        id: "spc_song_03_voice_1_pointer_hi",
        offset: 0x70a87,
        length: 1,
    }, // region-497
    Recipe {
        id: "spc_song_04_voice_1_pointer_hi",
        offset: 0x70a88,
        length: 1,
    }, // region-498
    Recipe {
        id: "spc_song_05_12_voice_1_high",
        offset: 0x70a89,
        length: 8,
    }, // region-1152
    Recipe {
        id: "spc_song_13_voice_1_pointer_hi",
        offset: 0x70a91,
        length: 1,
    }, // region-499
    Recipe {
        id: "spc_song_14_voice_1_pointer_hi",
        offset: 0x70a92,
        length: 1,
    }, // region-500
    Recipe {
        id: "spc_song_15_voice_1_pointer_hi",
        offset: 0x70a93,
        length: 1,
    }, // region-501
    Recipe {
        id: "spc_reserved_voice1_high_slice_0e90",
        offset: 0x70a94,
        length: 16,
    }, // region-1354
    Recipe {
        id: "spc_song_00_voice_2_pointer_lo",
        offset: 0x70aa4,
        length: 1,
    }, // region-502
    Recipe {
        id: "spc_song_01_voice_2_pointer_lo",
        offset: 0x70aa5,
        length: 1,
    }, // region-503
    Recipe {
        id: "spc_song_02_voice_2_pointer_lo",
        offset: 0x70aa6,
        length: 1,
    }, // region-504
    Recipe {
        id: "spc_song_03_voice_2_pointer_lo",
        offset: 0x70aa7,
        length: 1,
    }, // region-505
    Recipe {
        id: "spc_song_04_voice_2_pointer_lo",
        offset: 0x70aa8,
        length: 1,
    }, // region-506
    Recipe {
        id: "spc_song_05_12_voice_2_low",
        offset: 0x70aa9,
        length: 8,
    }, // region-1153
    Recipe {
        id: "spc_song_13_voice_2_pointer_lo",
        offset: 0x70ab1,
        length: 1,
    }, // region-507
    Recipe {
        id: "spc_song_14_voice_2_pointer_lo",
        offset: 0x70ab2,
        length: 1,
    }, // region-508
    Recipe {
        id: "spc_song_15_voice_2_pointer_lo",
        offset: 0x70ab3,
        length: 1,
    }, // region-509
    Recipe {
        id: "spc_reserved_voice2_low_slice_0eb0",
        offset: 0x70ab4,
        length: 16,
    }, // region-1355
    Recipe {
        id: "spc_song_00_voice_2_pointer_hi",
        offset: 0x70ac4,
        length: 1,
    }, // region-510
    Recipe {
        id: "spc_song_01_voice_2_pointer_hi",
        offset: 0x70ac5,
        length: 1,
    }, // region-511
    Recipe {
        id: "spc_song_02_voice_2_pointer_hi",
        offset: 0x70ac6,
        length: 1,
    }, // region-512
    Recipe {
        id: "spc_song_03_voice_2_pointer_hi",
        offset: 0x70ac7,
        length: 1,
    }, // region-513
    Recipe {
        id: "spc_song_04_voice_2_pointer_hi",
        offset: 0x70ac8,
        length: 1,
    }, // region-514
    Recipe {
        id: "spc_song_05_12_voice_2_high",
        offset: 0x70ac9,
        length: 8,
    }, // region-1154
    Recipe {
        id: "spc_song_13_voice_2_pointer_hi",
        offset: 0x70ad1,
        length: 1,
    }, // region-515
    Recipe {
        id: "spc_song_14_voice_2_pointer_hi",
        offset: 0x70ad2,
        length: 1,
    }, // region-516
    Recipe {
        id: "spc_song_15_voice_2_pointer_hi",
        offset: 0x70ad3,
        length: 1,
    }, // region-517
    Recipe {
        id: "spc_reserved_voice2_high_slice_0ed0",
        offset: 0x70ad4,
        length: 16,
    }, // region-1356
    Recipe {
        id: "spc_song_00_voice_3_pointer_lo",
        offset: 0x70ae4,
        length: 1,
    }, // region-518
    Recipe {
        id: "spc_song_01_voice_3_pointer_lo",
        offset: 0x70ae5,
        length: 1,
    }, // region-519
    Recipe {
        id: "spc_song_02_voice_3_pointer_lo",
        offset: 0x70ae6,
        length: 1,
    }, // region-520
    Recipe {
        id: "spc_song_03_voice_3_pointer_lo",
        offset: 0x70ae7,
        length: 1,
    }, // region-521
    Recipe {
        id: "spc_song_04_voice_3_pointer_lo",
        offset: 0x70ae8,
        length: 1,
    }, // region-522
    Recipe {
        id: "spc_song_05_12_voice_3_low",
        offset: 0x70ae9,
        length: 8,
    }, // region-1155
    Recipe {
        id: "spc_song_13_voice_3_pointer_lo",
        offset: 0x70af1,
        length: 1,
    }, // region-523
    Recipe {
        id: "spc_song_14_voice_3_pointer_lo",
        offset: 0x70af2,
        length: 1,
    }, // region-524
    Recipe {
        id: "spc_song_15_voice_3_pointer_lo",
        offset: 0x70af3,
        length: 1,
    }, // region-525
    Recipe {
        id: "spc_reserved_voice3_low_slice_0ef0",
        offset: 0x70af4,
        length: 16,
    }, // region-1357
    Recipe {
        id: "spc_song_00_voice_3_pointer_hi",
        offset: 0x70b04,
        length: 1,
    }, // region-526
    Recipe {
        id: "spc_song_01_voice_3_pointer_hi",
        offset: 0x70b05,
        length: 1,
    }, // region-527
    Recipe {
        id: "spc_song_02_voice_3_pointer_hi",
        offset: 0x70b06,
        length: 1,
    }, // region-528
    Recipe {
        id: "spc_song_03_voice_3_pointer_hi",
        offset: 0x70b07,
        length: 1,
    }, // region-529
    Recipe {
        id: "spc_song_04_voice_3_pointer_hi",
        offset: 0x70b08,
        length: 1,
    }, // region-530
    Recipe {
        id: "spc_song_05_12_voice_3_high",
        offset: 0x70b09,
        length: 8,
    }, // region-1156
    Recipe {
        id: "spc_song_13_voice_3_pointer_hi",
        offset: 0x70b11,
        length: 1,
    }, // region-531
    Recipe {
        id: "spc_song_14_voice_3_pointer_hi",
        offset: 0x70b12,
        length: 1,
    }, // region-532
    Recipe {
        id: "spc_song_15_voice_3_pointer_hi",
        offset: 0x70b13,
        length: 1,
    }, // region-533
    Recipe {
        id: "spc_reserved_voice3_high_slice_0f10",
        offset: 0x70b14,
        length: 16,
    }, // region-1358
    Recipe {
        id: "spc_song_00_voice_4_pointer_lo",
        offset: 0x70b24,
        length: 1,
    }, // region-534
    Recipe {
        id: "spc_song_01_voice_4_pointer_lo",
        offset: 0x70b25,
        length: 1,
    }, // region-535
    Recipe {
        id: "spc_song_02_voice_4_pointer_lo",
        offset: 0x70b26,
        length: 1,
    }, // region-536
    Recipe {
        id: "spc_song_03_voice_4_pointer_lo",
        offset: 0x70b27,
        length: 1,
    }, // region-537
    Recipe {
        id: "spc_song_04_voice_4_pointer_lo",
        offset: 0x70b28,
        length: 1,
    }, // region-538
    Recipe {
        id: "spc_song_05_12_voice_4_low",
        offset: 0x70b29,
        length: 8,
    }, // region-1157
    Recipe {
        id: "spc_song_13_voice_4_pointer_lo",
        offset: 0x70b31,
        length: 1,
    }, // region-539
    Recipe {
        id: "spc_song_14_voice_4_pointer_lo",
        offset: 0x70b32,
        length: 1,
    }, // region-540
    Recipe {
        id: "spc_song_15_voice_4_pointer_lo",
        offset: 0x70b33,
        length: 1,
    }, // region-541
    Recipe {
        id: "spc_reserved_voice4_low_slice_0f30",
        offset: 0x70b34,
        length: 16,
    }, // region-1359
    Recipe {
        id: "spc_song_00_voice_4_pointer_hi",
        offset: 0x70b44,
        length: 1,
    }, // region-542
    Recipe {
        id: "spc_song_01_voice_4_pointer_hi",
        offset: 0x70b45,
        length: 1,
    }, // region-543
    Recipe {
        id: "spc_song_02_voice_4_pointer_hi",
        offset: 0x70b46,
        length: 1,
    }, // region-544
    Recipe {
        id: "spc_song_03_voice_4_pointer_hi",
        offset: 0x70b47,
        length: 1,
    }, // region-545
    Recipe {
        id: "spc_song_04_voice_4_pointer_hi",
        offset: 0x70b48,
        length: 1,
    }, // region-546
    Recipe {
        id: "spc_song_05_12_voice_4_high",
        offset: 0x70b49,
        length: 8,
    }, // region-1158
    Recipe {
        id: "spc_song_13_voice_4_pointer_hi",
        offset: 0x70b51,
        length: 1,
    }, // region-547
    Recipe {
        id: "spc_song_14_voice_4_pointer_hi",
        offset: 0x70b52,
        length: 1,
    }, // region-548
    Recipe {
        id: "spc_song_15_voice_4_pointer_hi",
        offset: 0x70b53,
        length: 1,
    }, // region-549
    Recipe {
        id: "spc_reserved_voice4_high_slice_0f50",
        offset: 0x70b54,
        length: 16,
    }, // region-1360
    Recipe {
        id: "spc_song_00_voice_5_pointer_lo",
        offset: 0x70b64,
        length: 1,
    }, // region-550
    Recipe {
        id: "spc_song_01_voice_5_pointer_lo",
        offset: 0x70b65,
        length: 1,
    }, // region-551
    Recipe {
        id: "spc_song_02_voice_5_pointer_lo",
        offset: 0x70b66,
        length: 1,
    }, // region-552
    Recipe {
        id: "spc_song_03_voice_5_pointer_lo",
        offset: 0x70b67,
        length: 1,
    }, // region-553
    Recipe {
        id: "spc_song_04_voice_5_pointer_lo",
        offset: 0x70b68,
        length: 1,
    }, // region-554
    Recipe {
        id: "spc_song_05_12_voice_5_low",
        offset: 0x70b69,
        length: 8,
    }, // region-1159
    Recipe {
        id: "spc_song_13_voice_5_pointer_lo",
        offset: 0x70b71,
        length: 1,
    }, // region-555
    Recipe {
        id: "spc_song_14_voice_5_pointer_lo",
        offset: 0x70b72,
        length: 1,
    }, // region-556
    Recipe {
        id: "spc_song_15_voice_5_pointer_lo",
        offset: 0x70b73,
        length: 1,
    }, // region-557
    Recipe {
        id: "spc_reserved_voice5_low_slice_0f70",
        offset: 0x70b74,
        length: 16,
    }, // region-1361
    Recipe {
        id: "spc_song_00_voice_5_pointer_hi",
        offset: 0x70b84,
        length: 1,
    }, // region-558
    Recipe {
        id: "spc_song_01_voice_5_pointer_hi",
        offset: 0x70b85,
        length: 1,
    }, // region-559
    Recipe {
        id: "spc_song_02_voice_5_pointer_hi",
        offset: 0x70b86,
        length: 1,
    }, // region-560
    Recipe {
        id: "spc_song_03_voice_5_pointer_hi",
        offset: 0x70b87,
        length: 1,
    }, // region-561
    Recipe {
        id: "spc_song_04_voice_5_pointer_hi",
        offset: 0x70b88,
        length: 1,
    }, // region-562
    Recipe {
        id: "spc_song_05_12_voice_5_high",
        offset: 0x70b89,
        length: 8,
    }, // region-1160
    Recipe {
        id: "spc_song_13_voice_5_pointer_hi",
        offset: 0x70b91,
        length: 1,
    }, // region-563
    Recipe {
        id: "spc_song_14_voice_5_pointer_hi",
        offset: 0x70b92,
        length: 1,
    }, // region-564
    Recipe {
        id: "spc_song_15_voice_5_pointer_hi",
        offset: 0x70b93,
        length: 1,
    }, // region-565
    Recipe {
        id: "spc_reserved_voice5_high_slice_0f90",
        offset: 0x70b94,
        length: 16,
    }, // region-1362
    Recipe {
        id: "spc_song_00_voice_6_pointer_lo",
        offset: 0x70ba4,
        length: 1,
    }, // region-566
    Recipe {
        id: "spc_song_01_voice_6_pointer_lo",
        offset: 0x70ba5,
        length: 1,
    }, // region-567
    Recipe {
        id: "spc_song_02_voice_6_pointer_lo",
        offset: 0x70ba6,
        length: 1,
    }, // region-568
    Recipe {
        id: "spc_song_03_voice_6_pointer_lo",
        offset: 0x70ba7,
        length: 1,
    }, // region-569
    Recipe {
        id: "spc_song_04_voice_6_pointer_lo",
        offset: 0x70ba8,
        length: 1,
    }, // region-570
    Recipe {
        id: "spc_song_05_12_voice_6_low",
        offset: 0x70ba9,
        length: 8,
    }, // region-1161
    Recipe {
        id: "spc_song_13_voice_6_pointer_lo",
        offset: 0x70bb1,
        length: 1,
    }, // region-571
    Recipe {
        id: "spc_song_14_voice_6_pointer_lo",
        offset: 0x70bb2,
        length: 1,
    }, // region-572
    Recipe {
        id: "spc_song_15_voice_6_pointer_lo",
        offset: 0x70bb3,
        length: 1,
    }, // region-573
    Recipe {
        id: "spc_reserved_voice6_low_slice_0fb0",
        offset: 0x70bb4,
        length: 16,
    }, // region-1363
    Recipe {
        id: "spc_song_00_voice_6_pointer_hi",
        offset: 0x70bc4,
        length: 1,
    }, // region-574
    Recipe {
        id: "spc_song_01_voice_6_pointer_hi",
        offset: 0x70bc5,
        length: 1,
    }, // region-575
    Recipe {
        id: "spc_song_02_voice_6_pointer_hi",
        offset: 0x70bc6,
        length: 1,
    }, // region-576
    Recipe {
        id: "spc_song_03_voice_6_pointer_hi",
        offset: 0x70bc7,
        length: 1,
    }, // region-577
    Recipe {
        id: "spc_song_04_voice_6_pointer_hi",
        offset: 0x70bc8,
        length: 1,
    }, // region-578
    Recipe {
        id: "spc_song_05_12_voice_6_high",
        offset: 0x70bc9,
        length: 8,
    }, // region-1162
    Recipe {
        id: "spc_song_13_voice_6_pointer_hi",
        offset: 0x70bd1,
        length: 1,
    }, // region-579
    Recipe {
        id: "spc_song_14_voice_6_pointer_hi",
        offset: 0x70bd2,
        length: 1,
    }, // region-580
    Recipe {
        id: "spc_song_15_voice_6_pointer_hi",
        offset: 0x70bd3,
        length: 1,
    }, // region-581
    Recipe {
        id: "spc_reserved_voice6_high_slice_0fd0",
        offset: 0x70bd4,
        length: 16,
    }, // region-1364
    Recipe {
        id: "spc_song_00_voice_7_pointer_lo",
        offset: 0x70be4,
        length: 1,
    }, // region-582
    Recipe {
        id: "spc_song_01_voice_7_pointer_lo",
        offset: 0x70be5,
        length: 1,
    }, // region-583
    Recipe {
        id: "spc_song_02_voice_7_pointer_lo",
        offset: 0x70be6,
        length: 1,
    }, // region-584
    Recipe {
        id: "spc_song_03_voice_7_pointer_lo",
        offset: 0x70be7,
        length: 1,
    }, // region-585
    Recipe {
        id: "spc_song_04_voice_7_pointer_lo",
        offset: 0x70be8,
        length: 1,
    }, // region-586
    Recipe {
        id: "spc_song_05_12_voice_7_low",
        offset: 0x70be9,
        length: 8,
    }, // region-1163
    Recipe {
        id: "spc_song_13_voice_7_pointer_lo",
        offset: 0x70bf1,
        length: 1,
    }, // region-587
    Recipe {
        id: "spc_song_14_voice_7_pointer_lo",
        offset: 0x70bf2,
        length: 1,
    }, // region-588
    Recipe {
        id: "spc_song_15_voice_7_pointer_lo",
        offset: 0x70bf3,
        length: 1,
    }, // region-589
    Recipe {
        id: "spc_reserved_voice7_low_slice_0ff0",
        offset: 0x70bf4,
        length: 16,
    }, // region-1365
    Recipe {
        id: "spc_song_00_voice_7_pointer_hi",
        offset: 0x70c04,
        length: 1,
    }, // region-590
    Recipe {
        id: "spc_song_01_voice_7_pointer_hi",
        offset: 0x70c05,
        length: 1,
    }, // region-591
    Recipe {
        id: "spc_song_02_voice_7_pointer_hi",
        offset: 0x70c06,
        length: 1,
    }, // region-592
    Recipe {
        id: "spc_song_03_voice_7_pointer_hi",
        offset: 0x70c07,
        length: 1,
    }, // region-593
    Recipe {
        id: "spc_song_04_voice_7_pointer_hi",
        offset: 0x70c08,
        length: 1,
    }, // region-594
    Recipe {
        id: "spc_song_05_12_voice_7_high",
        offset: 0x70c09,
        length: 8,
    }, // region-1164
    Recipe {
        id: "spc_song_13_voice_7_pointer_hi",
        offset: 0x70c11,
        length: 1,
    }, // region-595
    Recipe {
        id: "spc_song_14_voice_7_pointer_hi",
        offset: 0x70c12,
        length: 1,
    }, // region-596
    Recipe {
        id: "spc_song_15_voice_7_pointer_hi",
        offset: 0x70c13,
        length: 1,
    }, // region-597
    Recipe {
        id: "spc_reserved_voice7_high_slice_1010",
        offset: 0x70c14,
        length: 16,
    }, // region-1366
    Recipe {
        id: "spc_song_01_config_pointer_lo",
        offset: 0x70c24,
        length: 1,
    }, // region-598
    Recipe {
        id: "spc_song_02_config_pointer_lo",
        offset: 0x70c25,
        length: 1,
    }, // region-599
    Recipe {
        id: "spc_song_03_config_pointer_lo",
        offset: 0x70c26,
        length: 1,
    }, // region-600
    Recipe {
        id: "spc_song_04_config_pointer_lo",
        offset: 0x70c27,
        length: 1,
    }, // region-601
    Recipe {
        id: "spc_song_05_12_configuration_low",
        offset: 0x70c28,
        length: 8,
    }, // region-1165
    Recipe {
        id: "spc_song_13_config_pointer_lo",
        offset: 0x70c30,
        length: 1,
    }, // region-602
    Recipe {
        id: "spc_song_14_config_pointer_lo",
        offset: 0x70c31,
        length: 1,
    }, // region-603
    Recipe {
        id: "spc_song_15_config_pointer_lo",
        offset: 0x70c32,
        length: 1,
    }, // region-604
    Recipe {
        id: "spc_reserved_config_low_slice_102f",
        offset: 0x70c33,
        length: 17,
    }, // region-1367
    Recipe {
        id: "spc_song_01_config_pointer_hi",
        offset: 0x70c44,
        length: 1,
    }, // region-605
    Recipe {
        id: "spc_song_02_config_pointer_hi",
        offset: 0x70c45,
        length: 1,
    }, // region-606
    Recipe {
        id: "spc_song_03_config_pointer_hi",
        offset: 0x70c46,
        length: 1,
    }, // region-607
    Recipe {
        id: "spc_song_04_config_pointer_hi",
        offset: 0x70c47,
        length: 1,
    }, // region-608
    Recipe {
        id: "spc_song_05_12_configuration_high",
        offset: 0x70c48,
        length: 8,
    }, // region-1166
    Recipe {
        id: "spc_song_13_config_pointer_hi",
        offset: 0x70c50,
        length: 1,
    }, // region-609
    Recipe {
        id: "spc_song_14_config_pointer_hi",
        offset: 0x70c51,
        length: 1,
    }, // region-610
    Recipe {
        id: "spc_song_15_config_pointer_hi",
        offset: 0x70c52,
        length: 1,
    }, // region-611
    Recipe {
        id: "spc_reserved_config_high_slice_104f",
        offset: 0x70c53,
        length: 17,
    }, // region-1368
    Recipe {
        id: "spc_effect_pointer_low_slice_1060",
        offset: 0x70c64,
        length: 2,
    }, // region-1332
    Recipe {
        id: "spc_effect_02_low",
        offset: 0x70c66,
        length: 1,
    }, // region-857
    Recipe {
        id: "spc_effect_03_low",
        offset: 0x70c67,
        length: 1,
    }, // region-858
    Recipe {
        id: "spc_effect_pointer_low_slice_1064",
        offset: 0x70c68,
        length: 1,
    }, // region-1333
    Recipe {
        id: "spc_effect_05_low",
        offset: 0x70c69,
        length: 1,
    }, // region-859
    Recipe {
        id: "spc_effect_06_low",
        offset: 0x70c6a,
        length: 1,
    }, // region-860
    Recipe {
        id: "spc_effect_07_low",
        offset: 0x70c6b,
        length: 1,
    }, // region-861
    Recipe {
        id: "spc_effect_08_low",
        offset: 0x70c6c,
        length: 1,
    }, // region-862
    Recipe {
        id: "spc_effect_pointer_low_slice_1069",
        offset: 0x70c6d,
        length: 7,
    }, // region-1334
    Recipe {
        id: "spc_effect_pointer_high_slice_1070",
        offset: 0x70c74,
        length: 2,
    }, // region-1335
    Recipe {
        id: "spc_effect_02_high",
        offset: 0x70c76,
        length: 1,
    }, // region-863
    Recipe {
        id: "spc_effect_03_high",
        offset: 0x70c77,
        length: 1,
    }, // region-864
    Recipe {
        id: "spc_effect_pointer_high_slice_1074",
        offset: 0x70c78,
        length: 1,
    }, // region-1336
    Recipe {
        id: "spc_effect_05_high",
        offset: 0x70c79,
        length: 1,
    }, // region-865
    Recipe {
        id: "spc_effect_06_high",
        offset: 0x70c7a,
        length: 1,
    }, // region-866
    Recipe {
        id: "spc_effect_07_high",
        offset: 0x70c7b,
        length: 1,
    }, // region-867
    Recipe {
        id: "spc_effect_08_high",
        offset: 0x70c7c,
        length: 1,
    }, // region-868
    Recipe {
        id: "spc_effect_pointer_high_slice_1079",
        offset: 0x70c7d,
        length: 7,
    }, // region-1337
    Recipe {
        id: "spc_effect_priority_slice_1080",
        offset: 0x70c84,
        length: 2,
    }, // region-1338
    Recipe {
        id: "spc_effect_02_priority",
        offset: 0x70c86,
        length: 1,
    }, // region-869
    Recipe {
        id: "spc_effect_03_priority",
        offset: 0x70c87,
        length: 1,
    }, // region-870
    Recipe {
        id: "spc_effect_priority_slice_1084",
        offset: 0x70c88,
        length: 1,
    }, // region-1339
    Recipe {
        id: "spc_effect_05_priority",
        offset: 0x70c89,
        length: 1,
    }, // region-871
    Recipe {
        id: "spc_effect_06_priority",
        offset: 0x70c8a,
        length: 1,
    }, // region-872
    Recipe {
        id: "spc_effect_07_priority",
        offset: 0x70c8b,
        length: 1,
    }, // region-873
    Recipe {
        id: "spc_effect_08_priority",
        offset: 0x70c8c,
        length: 1,
    }, // region-874
    Recipe {
        id: "spc_effect_priority_slice_1089",
        offset: 0x70c8d,
        length: 7,
    }, // region-1340
    Recipe {
        id: "spc_default_envelope_record",
        offset: 0x70c94,
        length: 7,
    }, // region-1343
    Recipe {
        id: "spc_initial_song_configuration_workspace",
        offset: 0x70c9b,
        length: 141,
    }, // region-1344
    Recipe {
        id: "spc_initial_envelope_record21_tail_and_later",
        offset: 0x70d28,
        length: 48,
    }, // region-1345
    Recipe {
        id: "spc_configuration_slice_1154",
        offset: 0x70d58,
        length: 105,
    }, // region-612
    Recipe {
        id: "spc_configuration_slice_11bd",
        offset: 0x70dc1,
        length: 36,
    }, // region-613
    Recipe {
        id: "spc_configuration_slice_11e1",
        offset: 0x70de5,
        length: 48,
    }, // region-614
    Recipe {
        id: "spc_configuration_slice_1211",
        offset: 0x70e15,
        length: 57,
    }, // region-615
    Recipe {
        id: "spc_configuration_slice_124a",
        offset: 0x70e4e,
        length: 6,
    }, // region-616
    Recipe {
        id: "spc_configuration_slice_1250",
        offset: 0x70e54,
        length: 78,
    }, // region-617
    Recipe {
        id: "spc_configuration_slice_129e",
        offset: 0x70ea2,
        length: 63,
    }, // region-618
    Recipe {
        id: "spc_song_configuration_slice_12dd",
        offset: 0x70ee1,
        length: 398,
    }, // region-1167
    Recipe {
        id: "spc_configuration_slice_146b",
        offset: 0x7106f,
        length: 141,
    }, // region-619
    Recipe {
        id: "spc_song_configuration_slice_14f8",
        offset: 0x710fc,
        length: 20,
    }, // region-1168
    Recipe {
        id: "spc_configuration_slice_150c",
        offset: 0x71110,
        length: 42,
    }, // region-620
    Recipe {
        id: "spc_configuration_slice_1536",
        offset: 0x7113a,
        length: 49,
    }, // region-621
    Recipe {
        id: "spc_song_0_sequence_slice_1567",
        offset: 0x7116b,
        length: 1,
    }, // region-622
    Recipe {
        id: "spc_song_1_sequence_slice_1568",
        offset: 0x7116c,
        length: 49,
    }, // region-623
    Recipe {
        id: "spc_song_1_sequence_slice_1599",
        offset: 0x7119d,
        length: 30,
    }, // region-624
    Recipe {
        id: "spc_configuration_slice_15b7",
        offset: 0x711bb,
        length: 4,
    }, // region-625
    Recipe {
        id: "spc_song_1_sequence_slice_15bb",
        offset: 0x711bf,
        length: 8,
    }, // region-626
    Recipe {
        id: "spc_song_1_sequence_slice_15c3",
        offset: 0x711c7,
        length: 134,
    }, // region-627
    Recipe {
        id: "unresolved-data-region-1458",
        offset: 0x7124d,
        length: 1,
    }, // region-1458
    Recipe {
        id: "spc_song_1_sequence_slice_164a",
        offset: 0x7124e,
        length: 37,
    }, // region-628
    Recipe {
        id: "unresolved-data-region-1459",
        offset: 0x71273,
        length: 1,
    }, // region-1459
    Recipe {
        id: "spc_song_1_sequence_slice_1670",
        offset: 0x71274,
        length: 55,
    }, // region-629
    Recipe {
        id: "unresolved-data-region-1460",
        offset: 0x712ab,
        length: 1,
    }, // region-1460
    Recipe {
        id: "spc_song_1_sequence_slice_16a8",
        offset: 0x712ac,
        length: 172,
    }, // region-630
    Recipe {
        id: "unresolved-data-region-1461",
        offset: 0x71358,
        length: 1,
    }, // region-1461
    Recipe {
        id: "spc_song_1_sequence_slice_1755",
        offset: 0x71359,
        length: 79,
    }, // region-631
    Recipe {
        id: "unresolved-data-region-1462",
        offset: 0x713a8,
        length: 1,
    }, // region-1462
    Recipe {
        id: "spc_song_1_sequence_slice_17a5",
        offset: 0x713a9,
        length: 97,
    }, // region-632
    Recipe {
        id: "unresolved-data-region-1463",
        offset: 0x7140a,
        length: 1,
    }, // region-1463
    Recipe {
        id: "spc_song_1_sequence_slice_1807",
        offset: 0x7140b,
        length: 51,
    }, // region-633
    Recipe {
        id: "conditional_voice_restart_transfer_183a",
        offset: 0x7143e,
        length: 5,
    }, // region-1372
    Recipe {
        id: "unresolved-data-region-1464",
        offset: 0x71443,
        length: 1,
    }, // region-1464
    Recipe {
        id: "spc_song_1_sequence_slice_1840",
        offset: 0x71444,
        length: 585,
    }, // region-634
    Recipe {
        id: "conditional_song_slur_phrase_1a89",
        offset: 0x7168d,
        length: 9,
    }, // region-1371
    Recipe {
        id: "spc_song_1_sequence_slice_1a92",
        offset: 0x71696,
        length: 176,
    }, // region-635
    Recipe {
        id: "spc_song_2_sequence_slice_1b42",
        offset: 0x71746,
        length: 74,
    }, // region-636
    Recipe {
        id: "unresolved-data-region-1465",
        offset: 0x71790,
        length: 1,
    }, // region-1465
    Recipe {
        id: "spc_song_2_sequence_slice_1b8d",
        offset: 0x71791,
        length: 136,
    }, // region-637
    Recipe {
        id: "unresolved-data-region-1466",
        offset: 0x71819,
        length: 1,
    }, // region-1466
    Recipe {
        id: "spc_song_2_sequence_slice_1c16",
        offset: 0x7181a,
        length: 133,
    }, // region-638
    Recipe {
        id: "unresolved-data-region-1467",
        offset: 0x7189f,
        length: 1,
    }, // region-1467
    Recipe {
        id: "spc_song_2_sequence_slice_1c9c",
        offset: 0x718a0,
        length: 21,
    }, // region-639
    Recipe {
        id: "unresolved-data-region-1468",
        offset: 0x718b5,
        length: 1,
    }, // region-1468
    Recipe {
        id: "spc_song_2_sequence_slice_1cb2",
        offset: 0x718b6,
        length: 161,
    }, // region-640
    Recipe {
        id: "unresolved-data-region-1469",
        offset: 0x71957,
        length: 1,
    }, // region-1469
    Recipe {
        id: "spc_song_2_sequence_slice_1d54",
        offset: 0x71958,
        length: 175,
    }, // region-641
    Recipe {
        id: "unresolved-data-region-1470",
        offset: 0x71a07,
        length: 3,
    }, // region-1470
    Recipe {
        id: "spc_song_2_sequence_slice_1e06",
        offset: 0x71a0a,
        length: 65,
    }, // region-642
    Recipe {
        id: "unresolved-data-region-1471",
        offset: 0x71a4b,
        length: 3,
    }, // region-1471
    Recipe {
        id: "spc_song_2_sequence_slice_1e4a",
        offset: 0x71a4e,
        length: 161,
    }, // region-643
    Recipe {
        id: "unresolved-data-region-1472",
        offset: 0x71aef,
        length: 1,
    }, // region-1472
    Recipe {
        id: "spc_song_3_sequence_slice_1eec",
        offset: 0x71af0,
        length: 727,
    }, // region-644
    Recipe {
        id: "spc_song_4_sequence_slice_21c3",
        offset: 0x71dc7,
        length: 54,
    }, // region-645
    Recipe {
        id: "unresolved-data-region-1473",
        offset: 0x71dfd,
        length: 1,
    }, // region-1473
    Recipe {
        id: "spc_song_4_sequence_slice_21fa",
        offset: 0x71dfe,
        length: 95,
    }, // region-646
    Recipe {
        id: "unresolved-data-region-1474",
        offset: 0x71e5d,
        length: 1,
    }, // region-1474
    Recipe {
        id: "spc_song_4_sequence_slice_225a",
        offset: 0x71e5e,
        length: 57,
    }, // region-647
    Recipe {
        id: "unresolved-data-region-1475",
        offset: 0x71e97,
        length: 1,
    }, // region-1475
    Recipe {
        id: "spc_song_4_sequence_slice_2294",
        offset: 0x71e98,
        length: 115,
    }, // region-648
    Recipe {
        id: "unresolved-data-region-1476",
        offset: 0x71f0b,
        length: 1,
    }, // region-1476
    Recipe {
        id: "spc_song_4_sequence_slice_2308",
        offset: 0x71f0c,
        length: 101,
    }, // region-649
    Recipe {
        id: "unresolved-data-region-1477",
        offset: 0x71f71,
        length: 1,
    }, // region-1477
    Recipe {
        id: "spc_song_4_sequence_slice_236e",
        offset: 0x71f72,
        length: 101,
    }, // region-650
    Recipe {
        id: "unresolved-data-region-1478",
        offset: 0x71fd7,
        length: 1,
    }, // region-1478
    Recipe {
        id: "spc_song_4_sequence_slice_23d4",
        offset: 0x71fd8,
        length: 2,
    }, // region-651
    Recipe {
        id: "spc_race_song_sequence_slice_23d6",
        offset: 0x71fda,
        length: 60,
    }, // region-1169
    Recipe {
        id: "unresolved-data-region-1479",
        offset: 0x72016,
        length: 1,
    }, // region-1479
    Recipe {
        id: "spc_race_song_sequence_slice_2413",
        offset: 0x72017,
        length: 134,
    }, // region-1170
    Recipe {
        id: "unresolved-data-region-1480",
        offset: 0x7209d,
        length: 1,
    }, // region-1480
    Recipe {
        id: "spc_race_song_sequence_slice_249a",
        offset: 0x7209e,
        length: 259,
    }, // region-1171
    Recipe {
        id: "unresolved-data-region-1481",
        offset: 0x721a1,
        length: 1,
    }, // region-1481
    Recipe {
        id: "spc_race_song_sequence_slice_259e",
        offset: 0x721a2,
        length: 39,
    }, // region-1172
    Recipe {
        id: "spc_race_song_sequence_slice_25c5",
        offset: 0x721c9,
        length: 18,
    }, // region-1173
    Recipe {
        id: "spc_race_song_sequence_slice_25d7",
        offset: 0x721db,
        length: 58,
    }, // region-1174
    Recipe {
        id: "unresolved-data-region-1482",
        offset: 0x72215,
        length: 1,
    }, // region-1482
    Recipe {
        id: "spc_race_song_sequence_slice_2612",
        offset: 0x72216,
        length: 141,
    }, // region-1175
    Recipe {
        id: "unresolved-data-region-1483",
        offset: 0x722a3,
        length: 1,
    }, // region-1483
    Recipe {
        id: "spc_race_song_sequence_slice_26a0",
        offset: 0x722a4,
        length: 95,
    }, // region-1176
    Recipe {
        id: "unresolved-data-region-1484",
        offset: 0x72303,
        length: 1,
    }, // region-1484
    Recipe {
        id: "spc_race_song_sequence_slice_2700",
        offset: 0x72304,
        length: 2,
    }, // region-1177
    Recipe {
        id: "spc_race_song_sequence_slice_2702",
        offset: 0x72306,
        length: 123,
    }, // region-1178
    Recipe {
        id: "unresolved-data-region-1485",
        offset: 0x72381,
        length: 1,
    }, // region-1485
    Recipe {
        id: "spc_race_song_sequence_slice_277e",
        offset: 0x72382,
        length: 81,
    }, // region-1179
    Recipe {
        id: "unresolved-data-region-1486",
        offset: 0x723d3,
        length: 1,
    }, // region-1486
    Recipe {
        id: "spc_race_song_sequence_slice_27d0",
        offset: 0x723d4,
        length: 48,
    }, // region-1180
    Recipe {
        id: "unresolved-data-region-1487",
        offset: 0x72404,
        length: 1,
    }, // region-1487
    Recipe {
        id: "spc_race_song_sequence_slice_2801",
        offset: 0x72405,
        length: 94,
    }, // region-1181
    Recipe {
        id: "unresolved-data-region-1488",
        offset: 0x72463,
        length: 1,
    }, // region-1488
    Recipe {
        id: "spc_race_song_sequence_slice_2860",
        offset: 0x72464,
        length: 65,
    }, // region-1182
    Recipe {
        id: "unresolved-data-region-1489",
        offset: 0x724a5,
        length: 1,
    }, // region-1489
    Recipe {
        id: "spc_race_song_sequence_slice_28a2",
        offset: 0x724a6,
        length: 63,
    }, // region-1183
    Recipe {
        id: "unresolved-data-region-1490",
        offset: 0x724e5,
        length: 1,
    }, // region-1490
    Recipe {
        id: "spc_race_song_sequence_slice_28e2",
        offset: 0x724e6,
        length: 192,
    }, // region-1184
    Recipe {
        id: "spc_song_15_sequence_slice_29a2",
        offset: 0x725a6,
        length: 36,
    }, // region-652
    Recipe {
        id: "spc_race_song_sequence_slice_29c6",
        offset: 0x725ca,
        length: 134,
    }, // region-1185
    Recipe {
        id: "unresolved-data-region-1491",
        offset: 0x72650,
        length: 1,
    }, // region-1491
    Recipe {
        id: "spc_race_song_sequence_slice_2a4d",
        offset: 0x72651,
        length: 199,
    }, // region-1186
    Recipe {
        id: "unresolved-data-region-1492",
        offset: 0x72718,
        length: 1,
    }, // region-1492
    Recipe {
        id: "spc_auxiliary_stream_2b15",
        offset: 0x72719,
        length: 19,
    }, // region-1187
    Recipe {
        id: "spc_race_song_sequence_slice_2b28",
        offset: 0x7272c,
        length: 162,
    }, // region-1188
    Recipe {
        id: "unresolved-data-region-1493",
        offset: 0x727ce,
        length: 1,
    }, // region-1493
    Recipe {
        id: "spc_race_song_sequence_slice_2bcb",
        offset: 0x727cf,
        length: 162,
    }, // region-1189
    Recipe {
        id: "unresolved-data-region-1494",
        offset: 0x72871,
        length: 1,
    }, // region-1494
    Recipe {
        id: "spc_race_song_sequence_slice_2c6e",
        offset: 0x72872,
        length: 81,
    }, // region-1190
    Recipe {
        id: "unresolved-data-region-1495",
        offset: 0x728c3,
        length: 1,
    }, // region-1495
    Recipe {
        id: "spc_race_song_sequence_slice_2cc0",
        offset: 0x728c4,
        length: 85,
    }, // region-1191
    Recipe {
        id: "unresolved-data-region-1496",
        offset: 0x72919,
        length: 1,
    }, // region-1496
    Recipe {
        id: "spc_race_song_sequence_slice_2d16",
        offset: 0x7291a,
        length: 2,
    }, // region-1192
    Recipe {
        id: "spc_race_song_sequence_slice_2d18",
        offset: 0x7291c,
        length: 62,
    }, // region-1193
    Recipe {
        id: "unresolved-data-region-1497",
        offset: 0x7295a,
        length: 1,
    }, // region-1497
    Recipe {
        id: "spc_race_song_sequence_slice_2d57",
        offset: 0x7295b,
        length: 35,
    }, // region-1194
    Recipe {
        id: "unresolved-data-region-1498",
        offset: 0x7297e,
        length: 1,
    }, // region-1498
    Recipe {
        id: "spc_race_song_sequence_slice_2d7b",
        offset: 0x7297f,
        length: 36,
    }, // region-1195
    Recipe {
        id: "unresolved-data-region-1499",
        offset: 0x729a3,
        length: 1,
    }, // region-1499
    Recipe {
        id: "spc_race_song_sequence_slice_2da0",
        offset: 0x729a4,
        length: 44,
    }, // region-1196
    Recipe {
        id: "unresolved-data-region-1500",
        offset: 0x729d0,
        length: 1,
    }, // region-1500
    Recipe {
        id: "spc_race_song_sequence_slice_2dcd",
        offset: 0x729d1,
        length: 293,
    }, // region-1197
    Recipe {
        id: "unresolved-data-region-1501",
        offset: 0x72af6,
        length: 1,
    }, // region-1501
    Recipe {
        id: "spc_race_song_sequence_slice_2ef3",
        offset: 0x72af7,
        length: 55,
    }, // region-1198
    Recipe {
        id: "unresolved-data-region-1502",
        offset: 0x72b2e,
        length: 1,
    }, // region-1502
    Recipe {
        id: "spc_race_song_sequence_slice_2f2b",
        offset: 0x72b2f,
        length: 152,
    }, // region-1199
    Recipe {
        id: "spc_race_song_sequence_slice_2fc3",
        offset: 0x72bc7,
        length: 135,
    }, // region-1200
    Recipe {
        id: "unresolved-data-region-1503",
        offset: 0x72c4e,
        length: 1,
    }, // region-1503
    Recipe {
        id: "spc_race_song_sequence_slice_304b",
        offset: 0x72c4f,
        length: 274,
    }, // region-1201
    Recipe {
        id: "unresolved-data-region-1504",
        offset: 0x72d61,
        length: 1,
    }, // region-1504
    Recipe {
        id: "spc_race_song_sequence_slice_315e",
        offset: 0x72d62,
        length: 9,
    }, // region-1202
    Recipe {
        id: "spc_auxiliary_stream_3167",
        offset: 0x72d6b,
        length: 19,
    }, // region-1203
    Recipe {
        id: "spc_race_song_sequence_slice_317a",
        offset: 0x72d7e,
        length: 289,
    }, // region-1204
    Recipe {
        id: "unresolved-data-region-1505",
        offset: 0x72e9f,
        length: 1,
    }, // region-1505
    Recipe {
        id: "spc_race_song_sequence_slice_329c",
        offset: 0x72ea0,
        length: 399,
    }, // region-1205
    Recipe {
        id: "spc_race_song_sequence_slice_342b",
        offset: 0x7302f,
        length: 110,
    }, // region-1206
    Recipe {
        id: "unresolved-data-region-1506",
        offset: 0x7309d,
        length: 1,
    }, // region-1506
    Recipe {
        id: "spc_race_song_sequence_slice_349a",
        offset: 0x7309e,
        length: 68,
    }, // region-1207
    Recipe {
        id: "unresolved-data-region-1507",
        offset: 0x730e2,
        length: 1,
    }, // region-1507
    Recipe {
        id: "spc_race_song_sequence_slice_34df",
        offset: 0x730e3,
        length: 41,
    }, // region-1208
    Recipe {
        id: "unresolved-data-region-1508",
        offset: 0x7310c,
        length: 1,
    }, // region-1508
    Recipe {
        id: "spc_race_song_sequence_slice_3509",
        offset: 0x7310d,
        length: 148,
    }, // region-1209
    Recipe {
        id: "unresolved-data-region-1509",
        offset: 0x731a1,
        length: 1,
    }, // region-1509
    Recipe {
        id: "spc_race_song_sequence_slice_359e",
        offset: 0x731a2,
        length: 149,
    }, // region-1210
    Recipe {
        id: "unresolved-data-region-1510",
        offset: 0x73237,
        length: 1,
    }, // region-1510
    Recipe {
        id: "spc_race_song_sequence_slice_3634",
        offset: 0x73238,
        length: 148,
    }, // region-1211
    Recipe {
        id: "unresolved-data-region-1511",
        offset: 0x732cc,
        length: 1,
    }, // region-1511
    Recipe {
        id: "spc_race_song_sequence_slice_36c9",
        offset: 0x732cd,
        length: 65,
    }, // region-1212
    Recipe {
        id: "spc_song_13_sequence_slice_370a",
        offset: 0x7330e,
        length: 116,
    }, // region-653
    Recipe {
        id: "spc_race_song_sequence_slice_377e",
        offset: 0x73382,
        length: 73,
    }, // region-1213
    Recipe {
        id: "unresolved-data-region-1512",
        offset: 0x733cb,
        length: 1,
    }, // region-1512
    Recipe {
        id: "spc_race_song_sequence_slice_37c8",
        offset: 0x733cc,
        length: 105,
    }, // region-1214
    Recipe {
        id: "unresolved-data-region-1513",
        offset: 0x73435,
        length: 1,
    }, // region-1513
    Recipe {
        id: "spc_race_song_sequence_slice_3832",
        offset: 0x73436,
        length: 176,
    }, // region-1215
    Recipe {
        id: "unresolved-data-region-1514",
        offset: 0x734e6,
        length: 1,
    }, // region-1514
    Recipe {
        id: "spc_race_song_sequence_slice_38e3",
        offset: 0x734e7,
        length: 199,
    }, // region-1216
    Recipe {
        id: "unresolved-data-region-1515",
        offset: 0x735ae,
        length: 1,
    }, // region-1515
    Recipe {
        id: "spc_race_song_sequence_slice_39ab",
        offset: 0x735af,
        length: 415,
    }, // region-1217
    Recipe {
        id: "unresolved-data-region-1516",
        offset: 0x7374e,
        length: 1,
    }, // region-1516
    Recipe {
        id: "spc_race_song_sequence_slice_3b4b",
        offset: 0x7374f,
        length: 97,
    }, // region-1218
    Recipe {
        id: "unresolved-data-region-1517",
        offset: 0x737b0,
        length: 1,
    }, // region-1517
    Recipe {
        id: "spc_race_song_sequence_slice_3bad",
        offset: 0x737b1,
        length: 47,
    }, // region-1219
    Recipe {
        id: "spc_race_song_sequence_slice_3bdc",
        offset: 0x737e0,
        length: 119,
    }, // region-1220
    Recipe {
        id: "unresolved-data-region-1518",
        offset: 0x73857,
        length: 1,
    }, // region-1518
    Recipe {
        id: "spc_race_song_sequence_slice_3c54",
        offset: 0x73858,
        length: 137,
    }, // region-1221
    Recipe {
        id: "unresolved-data-region-1519",
        offset: 0x738e1,
        length: 1,
    }, // region-1519
    Recipe {
        id: "spc_race_song_sequence_slice_3cde",
        offset: 0x738e2,
        length: 163,
    }, // region-1222
    Recipe {
        id: "unresolved-data-region-1520",
        offset: 0x73985,
        length: 1,
    }, // region-1520
    Recipe {
        id: "spc_race_song_sequence_slice_3d82",
        offset: 0x73986,
        length: 181,
    }, // region-1223
    Recipe {
        id: "unresolved-data-region-1521",
        offset: 0x73a3b,
        length: 1,
    }, // region-1521
    Recipe {
        id: "spc_race_song_sequence_slice_3e38",
        offset: 0x73a3c,
        length: 129,
    }, // region-1224
    Recipe {
        id: "unresolved-data-region-1522",
        offset: 0x73abd,
        length: 1,
    }, // region-1522
    Recipe {
        id: "spc_race_song_sequence_slice_3eba",
        offset: 0x73abe,
        length: 281,
    }, // region-1225
    Recipe {
        id: "unresolved-data-region-1523",
        offset: 0x73bd7,
        length: 1,
    }, // region-1523
    Recipe {
        id: "spc_race_song_sequence_slice_3fd4",
        offset: 0x73bd8,
        length: 221,
    }, // region-1226
    Recipe {
        id: "spc_song_15_sequence_slice_40b1",
        offset: 0x73cb5,
        length: 324,
    }, // region-654
    Recipe {
        id: "spc_song_14_sequence_slice_41f5",
        offset: 0x73df9,
        length: 322,
    }, // region-655
    Recipe {
        id: "spc_song_3_sequence_slice_4337",
        offset: 0x73f3b,
        length: 19,
    }, // region-656
    Recipe {
        id: "spc_effect_01_sequence",
        offset: 0x73f4e,
        length: 13,
    }, // region-1341
    Recipe {
        id: "spc_effect_bytes_073f5b",
        offset: 0x73f5b,
        length: 19,
    }, // region-875
    Recipe {
        id: "spc_effect_bytes_073f6e",
        offset: 0x73f6e,
        length: 17,
    }, // region-876
    Recipe {
        id: "spc_effect_04_sequence",
        offset: 0x73f7f,
        length: 17,
    }, // region-1342
    Recipe {
        id: "spc_effect_bytes_073f90",
        offset: 0x73f90,
        length: 23,
    }, // region-877
    Recipe {
        id: "spc_conditional_standalone_effect_sequence",
        offset: 0x73fa7,
        length: 23,
    }, // region-1346
    Recipe {
        id: "spc_effect_bytes_073fbe",
        offset: 0x73fbe,
        length: 53,
    }, // region-878
    Recipe {
        id: "spc_effect_bytes_073ff3",
        offset: 0x73ff3,
        length: 54,
    }, // region-879
    Recipe {
        id: "unresolved-data-region-1524",
        offset: 0x74029,
        length: 3035,
    }, // region-1524
    Recipe {
        id: "spc_sample_00_directory",
        offset: 0x74c04,
        length: 4,
    }, // region-657
    Recipe {
        id: "spc_sample_directory_slice_5004",
        offset: 0x74c08,
        length: 4,
    }, // region-1318
    Recipe {
        id: "spc_sample_02_directory",
        offset: 0x74c0c,
        length: 4,
    }, // region-658
    Recipe {
        id: "spc_sample_03_directory",
        offset: 0x74c10,
        length: 4,
    }, // region-659
    Recipe {
        id: "spc_sample_04_directory",
        offset: 0x74c14,
        length: 4,
    }, // region-1227
    Recipe {
        id: "spc_sample_05_directory",
        offset: 0x74c18,
        length: 4,
    }, // region-660
    Recipe {
        id: "spc_sample_06_directory",
        offset: 0x74c1c,
        length: 4,
    }, // region-1228
    Recipe {
        id: "spc_sample_07_directory",
        offset: 0x74c20,
        length: 4,
    }, // region-661
    Recipe {
        id: "spc_sample_08_directory",
        offset: 0x74c24,
        length: 4,
    }, // region-662
    Recipe {
        id: "spc_sample_directory_slice_5024",
        offset: 0x74c28,
        length: 4,
    }, // region-1319
    Recipe {
        id: "spc_sample_10_directory",
        offset: 0x74c2c,
        length: 4,
    }, // region-663
    Recipe {
        id: "spc_sample_11_directory",
        offset: 0x74c30,
        length: 4,
    }, // region-1229
    Recipe {
        id: "spc_sample_12_directory",
        offset: 0x74c34,
        length: 4,
    }, // region-664
    Recipe {
        id: "spc_sample_13_directory",
        offset: 0x74c38,
        length: 4,
    }, // region-665
    Recipe {
        id: "spc_sample_14_directory",
        offset: 0x74c3c,
        length: 4,
    }, // region-666
    Recipe {
        id: "spc_sample_15_directory",
        offset: 0x74c40,
        length: 4,
    }, // region-667
    Recipe {
        id: "spc_sample_16_directory",
        offset: 0x74c44,
        length: 4,
    }, // region-668
    Recipe {
        id: "spc_sample_directory_slice_5044",
        offset: 0x74c48,
        length: 12,
    }, // region-1320
    Recipe {
        id: "spc_sample_20_directory",
        offset: 0x74c54,
        length: 4,
    }, // region-669
    Recipe {
        id: "spc_sample_21_directory",
        offset: 0x74c58,
        length: 4,
    }, // region-670
    Recipe {
        id: "spc_sample_22_directory",
        offset: 0x74c5c,
        length: 4,
    }, // region-671
    Recipe {
        id: "spc_sample_23_directory",
        offset: 0x74c60,
        length: 4,
    }, // region-672
    Recipe {
        id: "spc_sample_24_directory",
        offset: 0x74c64,
        length: 4,
    }, // region-673
    Recipe {
        id: "spc_sample_25_directory",
        offset: 0x74c68,
        length: 4,
    }, // region-674
    Recipe {
        id: "spc_sample_directory_slice_5068",
        offset: 0x74c6c,
        length: 4,
    }, // region-1321
    Recipe {
        id: "spc_sample_27_directory",
        offset: 0x74c70,
        length: 4,
    }, // region-880
    Recipe {
        id: "spc_sample_28_directory",
        offset: 0x74c74,
        length: 4,
    }, // region-881
    Recipe {
        id: "spc_sample_29_directory",
        offset: 0x74c78,
        length: 4,
    }, // region-675
    Recipe {
        id: "spc_sample_30_directory",
        offset: 0x74c7c,
        length: 4,
    }, // region-676
    Recipe {
        id: "spc_sample_31_directory",
        offset: 0x74c80,
        length: 4,
    }, // region-677
    Recipe {
        id: "spc_sample_directory_slice_5080",
        offset: 0x74c84,
        length: 4,
    }, // region-1322
    Recipe {
        id: "spc_instrument_note_base_index_00_slice",
        offset: 0x74c88,
        length: 1,
    }, // region-1230
    Recipe {
        id: "spc_instrument_note_base_slice_5085",
        offset: 0x74c89,
        length: 1,
    }, // region-1323
    Recipe {
        id: "spc_instrument_note_base_index_02_slice",
        offset: 0x74c8a,
        length: 1,
    }, // region-1231
    Recipe {
        id: "spc_instrument_note_base_index_03_slice",
        offset: 0x74c8b,
        length: 6,
    }, // region-1232
    Recipe {
        id: "spc_instrument_note_base_slice_508d",
        offset: 0x74c91,
        length: 1,
    }, // region-1324
    Recipe {
        id: "spc_instrument_note_base_index_10_slice",
        offset: 0x74c92,
        length: 7,
    }, // region-1233
    Recipe {
        id: "spc_instrument_note_base_slice_5095",
        offset: 0x74c99,
        length: 3,
    }, // region-1325
    Recipe {
        id: "spc_instrument_note_base_index_20_slice",
        offset: 0x74c9c,
        length: 3,
    }, // region-1234
    Recipe {
        id: "spc_instrument_note_base_index_23_slice",
        offset: 0x74c9f,
        length: 3,
    }, // region-1235
    Recipe {
        id: "spc_instrument_note_base_slice_509e",
        offset: 0x74ca2,
        length: 1,
    }, // region-1326
    Recipe {
        id: "spc_instrument_27_note_base_cell",
        offset: 0x74ca3,
        length: 1,
    }, // region-1267
    Recipe {
        id: "spc_instrument_28_note_base_cell",
        offset: 0x74ca4,
        length: 1,
    }, // region-1268
    Recipe {
        id: "spc_instrument_note_base_index_29_slice",
        offset: 0x74ca5,
        length: 3,
    }, // region-1236
    Recipe {
        id: "spc_instrument_pitch_scale_index_00_slice",
        offset: 0x74ca8,
        length: 1,
    }, // region-1237
    Recipe {
        id: "spc_instrument_pitch_scale_slice_50a5",
        offset: 0x74ca9,
        length: 1,
    }, // region-1327
    Recipe {
        id: "spc_instrument_pitch_scale_index_02_slice",
        offset: 0x74caa,
        length: 1,
    }, // region-1238
    Recipe {
        id: "spc_instrument_pitch_scale_index_03_slice",
        offset: 0x74cab,
        length: 6,
    }, // region-1239
    Recipe {
        id: "spc_instrument_pitch_scale_slice_50ad",
        offset: 0x74cb1,
        length: 1,
    }, // region-1328
    Recipe {
        id: "spc_instrument_pitch_scale_index_10_slice",
        offset: 0x74cb2,
        length: 7,
    }, // region-1240
    Recipe {
        id: "spc_instrument_pitch_scale_slice_50b5",
        offset: 0x74cb9,
        length: 3,
    }, // region-1329
    Recipe {
        id: "spc_instrument_pitch_scale_index_20_slice",
        offset: 0x74cbc,
        length: 3,
    }, // region-1241
    Recipe {
        id: "spc_instrument_pitch_scale_index_23_slice",
        offset: 0x74cbf,
        length: 3,
    }, // region-1242
    Recipe {
        id: "spc_instrument_pitch_scale_slice_50be",
        offset: 0x74cc2,
        length: 1,
    }, // region-1330
    Recipe {
        id: "spc_instrument_27_pitch_scale_cell",
        offset: 0x74cc3,
        length: 1,
    }, // region-1269
    Recipe {
        id: "spc_instrument_28_pitch_scale_cell",
        offset: 0x74cc4,
        length: 1,
    }, // region-1270
    Recipe {
        id: "spc_instrument_pitch_scale_index_29_slice",
        offset: 0x74cc5,
        length: 3,
    }, // region-1243
    Recipe {
        id: "spc_silent_loop_brr",
        offset: 0x74cc8,
        length: 9,
    }, // region-1331
    Recipe {
        id: "spc_sample_00_brr",
        offset: 0x74cd1,
        length: 279,
    }, // region-678
    Recipe {
        id: "spc_sample_02_brr",
        offset: 0x74de8,
        length: 153,
    }, // region-679
    Recipe {
        id: "spc_sample_03_brr",
        offset: 0x74e81,
        length: 90,
    }, // region-680
    Recipe {
        id: "spc_sample_04_brr",
        offset: 0x74edb,
        length: 432,
    }, // region-1244
    Recipe {
        id: "spc_sample_05_brr",
        offset: 0x7508b,
        length: 306,
    }, // region-681
    Recipe {
        id: "spc_sample_07_brr",
        offset: 0x751bd,
        length: 1413,
    }, // region-682
    Recipe {
        id: "spc_sample_08_brr",
        offset: 0x75742,
        length: 783,
    }, // region-683
    Recipe {
        id: "spc_sample_10_brr",
        offset: 0x75a51,
        length: 855,
    }, // region-684
    Recipe {
        id: "spc_sample_11_brr",
        offset: 0x75da8,
        length: 1827,
    }, // region-1245
    Recipe {
        id: "spc_sample_12_brr",
        offset: 0x764cb,
        length: 855,
    }, // region-685
    Recipe {
        id: "spc_sample_13_brr",
        offset: 0x76822,
        length: 108,
    }, // region-686
    Recipe {
        id: "spc_sample_14_brr",
        offset: 0x7688e,
        length: 108,
    }, // region-687
    Recipe {
        id: "spc_sample_15_brr",
        offset: 0x768fa,
        length: 3042,
    }, // region-688
    Recipe {
        id: "spc_sample_16_brr",
        offset: 0x774dc,
        length: 108,
    }, // region-689
    Recipe {
        id: "spc_sample_20_brr",
        offset: 0x77548,
        length: 2236,
    }, // region-690
    Recipe {
        id: "spc_upload_block1_terminator",
        offset: 0x77e04,
        length: 4,
    }, // region-89
    Recipe {
        id: "unresolved-data-region-1526",
        offset: 0x77e08,
        length: 504,
    }, // region-1526
    Recipe {
        id: "spc_upload_block2_header",
        offset: 0x78000,
        length: 4,
    }, // region-90
    Recipe {
        id: "spc_sample_20_brr_continued",
        offset: 0x78004,
        length: 1427,
    }, // region-691
    Recipe {
        id: "spc_sample_21_brr",
        offset: 0x78597,
        length: 1755,
    }, // region-692
    Recipe {
        id: "spc_sample_22_brr",
        offset: 0x78c72,
        length: 4509,
    }, // region-693
    Recipe {
        id: "spc_sample_23_brr",
        offset: 0x79e0f,
        length: 981,
    }, // region-694
    Recipe {
        id: "spc_sample_24_brr",
        offset: 0x7a1e4,
        length: 2088,
    }, // region-695
    Recipe {
        id: "spc_sample_25_brr",
        offset: 0x7aa0c,
        length: 4950,
    }, // region-696
    Recipe {
        id: "spc_sample_27_brr",
        offset: 0x7bd62,
        length: 2268,
    }, // region-882
    Recipe {
        id: "spc_sample_29_brr",
        offset: 0x7c63e,
        length: 3735,
    }, // region-697
    Recipe {
        id: "spc_sample_30_brr",
        offset: 0x7d4d5,
        length: 5094,
    }, // region-698
    Recipe {
        id: "spc_sample_31_brr",
        offset: 0x7e8bb,
        length: 4464,
    }, // region-699
    Recipe {
        id: "unresolved-data-region-1525",
        offset: 0x7fa2b,
        length: 985,
    }, // region-1525
    Recipe {
        id: "spc_upload_block2_terminator",
        offset: 0x7fe04,
        length: 4,
    }, // region-91
    Recipe {
        id: "unresolved-data-region-1527",
        offset: 0x7fe08,
        length: 504,
    }, // region-1527
];
