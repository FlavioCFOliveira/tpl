-- =============================================================================
-- tpl catalogue fixture — data
-- =============================================================================
--
-- Rows for every table in `freight`. The figures are the real ones where a real
-- one exists — UN/LOCODEs, IMO numbers, published container throughput, quay
-- lengths, ISO 6346 equipment types — and plausible where the real one is
-- commercially confidential, as rates and credit limits are. The companies are
-- invented; the ports and the ships are not.
--
-- The awkward cases the DDL sets up are exercised here on purpose: the ENUM
-- member holding an apostrophe, the ENUM members holding commas, the SET, the
-- non-ASCII text, the "to order" consignment with no consignee, the
-- consignment with no voyage assigned, and NULLs wherever they are legal.
--
-- audit_event is not seeded. It is written by the triggers on consignment, and
-- the statements at the foot of this file drive an insert, an update and a
-- delete so that all six of them fire.
-- =============================================================================

SET NAMES utf8mb4;
USE freight;

-- -----------------------------------------------------------------------------
-- port
-- -----------------------------------------------------------------------------

INSERT INTO port
  (un_locode, name, country_code, local_name, timezone_name, utc_offset_minutes,
   berth_count, max_draught_m, tidal_range_m, annual_throughput_teu,
   customs_round_clock, harbour_dues_currency, approach_notes)
VALUES
  ('PTLIS', 'Lisbon',        'PT', 'Lisboa',      'Europe/Lisbon',    0,  12, 12.00, 3.8,    550000, b'0', 'EUR',
   'Enter via the Barra Sul channel. Pilotage compulsory above 100 m LOA; the Tagus ebb runs up to 4 knots at springs.'),
  ('PTLEI', 'Leixoes',       'PT', 'Leixões',     'Europe/Lisbon',    0,   8, 12.00, 3.2,    660000, b'0', 'EUR',
   'Narrow entrance between the north and south breakwaters. Swell restrictions apply from October to March.'),
  ('ESALG', 'Algeciras',     'ES', 'Algeciras',   'Europe/Madrid',   60,  14, 18.50, 1.1,   4760000, b'1', 'EUR',
   'Deep water throughout. Strong Levante winds close the container berths several days each year.'),
  ('NLRTM', 'Rotterdam',     'NL', 'Rotterdam',   'Europe/Amsterdam', 60,  42, 24.00, 1.9,  13450000, b'1', 'EUR',
   'Maasvlakte II accepts the largest vessels afloat without tidal window.'),
  ('BEANR', 'Antwerp',       'BE', 'Antwerpen',   'Europe/Brussels',  60,  36, 16.00, 5.2,  12500000, b'1', 'EUR',
   'Tidal window required through the Westerschelde; locks add two to four hours to the passage.'),
  ('DEHAM', 'Hamburg',       'DE', 'Hamburg',     'Europe/Berlin',    60,  30, 15.10, 3.6,   7800000, b'1', 'EUR',
   'One hundred kilometres up the Elbe; deep-draught vessels sail on the flood.'),
  ('GBFXT', 'Felixstowe',    'GB', 'Felixstowe',  'Europe/London',     0,  18, 16.00, 3.9,   3600000, b'1', 'GBP',
   'No lock. Trinity Terminal berths 8 and 9 take ultra-large vessels at any state of tide.'),
  ('MAPTM', 'Tanger Med',    'MA', 'طنجة المتوسط','Africa/Casablanca', 60,  16, 18.00, 1.4,   8600000, b'1', 'EUR',
   'Straits transhipment hub. Traffic separation scheme in force; keep clear of the westbound lane.'),
  ('SNDKR', 'Dakar',         'SN', 'Dakar',       'Africa/Dakar',      0,   9, 13.00, 1.2,    800000, b'0', 'XOF',
   'Approach from the west of Cap Manuel. Congestion is routine between November and February.'),
  ('CIABJ', 'Abidjan',       'CI', 'Abidjan',     'Africa/Abidjan',    0,  11, 13.50, 1.0,   1200000, b'0', 'XOF',
   'Vridi canal transit, one-way traffic, daylight only for vessels above 250 m LOA.'),
  ('NGLOS', 'Lagos',         'NG', 'Èkó',         'Africa/Lagos',     60,  10, 13.50, 1.1,   1100000, b'0', 'USD',
   'Apapa and Tin Can Island. Anchorage waiting times of two weeks are not unusual.'),
  ('BRSSZ', 'Santos',        'BR', 'Santos',      'America/Sao_Paulo', -180, 20, 15.00, 1.5,  4900000, b'1', 'USD',
   'Long approach channel with a strict one-way regime for the largest vessels.'),
  ('CNSHA', 'Shanghai',      'CN', '上海',         'Asia/Shanghai',    480,  60, 17.50, 4.6,  49300000, b'1', 'CNY',
   'Yangshan deep-water terminals reached by the Donghai bridge; Waigaoqiao is tide-restricted.'),
  ('SGSIN', 'Singapore',     'SG', 'Singapura',   'Asia/Singapore',   480,  55, 16.00, 2.6,  39000000, b'1', 'USD',
   'Tuas and Pasir Panjang. Bunkering is available at every berth.'),
  ('AEJEA', 'Jebel Ali',     'AE', 'جبل علي',      'Asia/Dubai',       240,  25, 17.00, 1.3,  14500000, b'1', 'USD',
   'Largest man-made harbour in the world; no tidal restriction on the container berths.'),
  ('NOALS', 'Alesund',       'NO', 'Ålesund',     'Europe/Oslo',       60,   3,  9.00, 1.8,     15000, b'0', 'NOK',
   'Feeder and reefer traffic only. The local name carries a ring above the A, which is why the column is binary.');

-- -----------------------------------------------------------------------------
-- port_facility
-- -----------------------------------------------------------------------------
-- Geometry is written in WKT as (longitude latitude), the order MariaDB's
-- ST_GeomFromText expects for these fixtures.

INSERT INTO port_facility
  (port_id, designation, facility_kind, quay_length_m, crane_count,
   gate_opens_at, gate_closes_at, berth_outline, approach_channel,
   anchorage_points, fairway_network, restricted_areas, reference_marker,
   survey_footprint, notices_to_mariners)
SELECT p.port_id, 'Alcantara Norte', 'Container terminal', 630.00, 6,
       '06:00:00', '22:00:00',
       ST_GeomFromText('POLYGON((-9.1780 38.6980, -9.1700 38.6980, -9.1700 38.7020, -9.1780 38.7020, -9.1780 38.6980))'),
       ST_GeomFromText('LINESTRING(-9.2400 38.6700, -9.2000 38.6850, -9.1780 38.6990)'),
       ST_GeomFromText('MULTIPOINT(-9.2500 38.6600, -9.2600 38.6550)'),
       ST_GeomFromText('MULTILINESTRING((-9.2400 38.6700, -9.1780 38.6990), (-9.2350 38.6650, -9.1900 38.6900))'),
       ST_GeomFromText('MULTIPOLYGON(((-9.2100 38.6800, -9.2050 38.6800, -9.2050 38.6830, -9.2100 38.6830, -9.2100 38.6800)))'),
       ST_GeomFromText('POINT(-9.1740 38.7000)'),
       ST_GeomFromText('POLYGON((-9.1790 38.6975, -9.1690 38.6975, -9.1690 38.7025, -9.1790 38.7025, -9.1790 38.6975))'),
       ST_GeomFromText('GEOMETRYCOLLECTION(POINT(-9.1760 38.6995), LINESTRING(-9.1800 38.6970, -9.1750 38.7010))')
  FROM port p WHERE p.un_locode = 'PTLIS';

INSERT INTO port_facility
  (port_id, designation, facility_kind, quay_length_m, crane_count,
   gate_opens_at, gate_closes_at, berth_outline, approach_channel,
   anchorage_points, reference_marker)
SELECT p.port_id, 'Leixoes Sul', 'Container terminal', 540.00, 4,
       '05:30:00', '23:30:00',
       ST_GeomFromText('POLYGON((-8.7000 41.1830, -8.6930 41.1830, -8.6930 41.1870, -8.7000 41.1870, -8.7000 41.1830))'),
       ST_GeomFromText('LINESTRING(-8.7200 41.1750, -8.7050 41.1800, -8.6960 41.1845)'),
       ST_GeomFromText('MULTIPOINT(-8.7300 41.1700, -8.7350 41.1680)'),
       ST_GeomFromText('POINT(-8.6960 41.1850)')
  FROM port p WHERE p.un_locode = 'PTLEI';

INSERT INTO port_facility
  (port_id, designation, facility_kind, quay_length_m, crane_count,
   gate_opens_at, gate_closes_at, berth_outline, fairway_network, restricted_areas)
SELECT p.port_id, 'Maasvlakte II — APM', 'Container terminal', 1000.00, 14,
       '00:00:00', '23:59:59',
       ST_GeomFromText('POLYGON((4.0100 51.9500, 4.0300 51.9500, 4.0300 51.9560, 4.0100 51.9560, 4.0100 51.9500))'),
       ST_GeomFromText('MULTILINESTRING((3.9500 51.9600, 4.0100 51.9530), (3.9400 51.9700, 4.0000 51.9580))'),
       ST_GeomFromText('MULTIPOLYGON(((3.9800 51.9620, 3.9850 51.9620, 3.9850 51.9650, 3.9800 51.9650, 3.9800 51.9620)))')
  FROM port p WHERE p.un_locode = 'NLRTM';

INSERT INTO port_facility
  (port_id, designation, facility_kind, quay_length_m, crane_count,
   gate_opens_at, gate_closes_at, berth_outline, approach_channel)
SELECT p.port_id, 'Juan Carlos I', 'Container terminal', 1900.00, 22,
       '00:00:00', '23:59:59',
       ST_GeomFromText('POLYGON((-5.4400 36.1350, -5.4300 36.1350, -5.4300 36.1420, -5.4400 36.1420, -5.4400 36.1350))'),
       ST_GeomFromText('LINESTRING(-5.4600 36.1200, -5.4480 36.1300, -5.4380 36.1380)')
  FROM port p WHERE p.un_locode = 'ESALG';

INSERT INTO port_facility
  (port_id, designation, facility_kind, quay_length_m, crane_count,
   gate_opens_at, gate_closes_at, berth_outline, reference_marker)
SELECT p.port_id, 'Skansekaia', 'Reefer terminal', 180.00, 1,
       '07:00:00', '17:00:00',
       ST_GeomFromText('POLYGON((6.1520 62.4720, 6.1560 62.4720, 6.1560 62.4740, 6.1520 62.4740, 6.1520 62.4720))'),
       ST_GeomFromText('POINT(6.1540 62.4730)')
  FROM port p WHERE p.un_locode = 'NOALS';

-- -----------------------------------------------------------------------------
-- vessel
-- -----------------------------------------------------------------------------
-- Real ships, real IMO numbers. Elbe Trader is out of service, so that
-- v_active_vessel has something to exclude.

INSERT INTO vessel
  (imo_number, name, call_sign, mmsi, flag_state, class_society, hull_number,
   capacity_teu, reefer_plugs, deadweight_tonnes, gross_tonnage, net_tonnage,
   service_speed_knots, ballast_capacity_m3, loa_m, beam_m, design_draught_m,
   year_built, crew_complement, lifetime_distance_nm, in_service,
   scrubber_fitted, particulars_sheet)
VALUES
  ('9839430', 'MSC Gülsün', '3EWU8', 373817000, 'PA', 'Lloyd''s Register', 2415,
   23756, 2000, 228149, 232618.00, 108734.00, 22.0, 84500.0, 399.90, 61.50, 16.00,
   2019, 26,  742180, TRUE, b'1',
   'Twenty-three thousand TEU class. Twenty-four rows on deck. Fitted with an open-loop exhaust gas cleaning system.'),
  ('9893890', 'Ever Ace', '3FQE9', 353136000, 'PA', 'ClassNK', 1620,
   23992, 1000, 235579, 235579.00, 110000.00, 22.6, 79800.0, 399.90, 61.53, 17.00,
   2021, 25,  418960, TRUE, b'0',
   'A-class. Longest container ship in service at delivery. Twin-island arrangement.'),
  ('9839179', 'CMA CGM Jacques Saadé', 'FLSV', 228389800, 'FR', 'Bureau Veritas', 1481,
   23112, 2100, 220766, 236583.00, 112400.00, 22.0, 81200.0, 399.90, 61.30, 16.00,
   2020, 27,  622540, TRUE, b'0',
   'First 23 000 TEU vessel powered by liquefied natural gas. Membrane tanks of 18 600 cubic metres.'),
  ('9967494', 'Berlin Express', 'A8YZ7', 636023119, 'DE', 'DNV', 1408,
   23664, 2200, 222500, 236428.00, 109900.00, 22.5, 82000.0, 399.90, 61.40, 16.50,
   2023, 24,  148300, TRUE, b'0',
   'Dual-fuel methane. Delivered from Hanwha Ocean, Geoje.'),
  ('9784271', 'Maersk Sentosa', '9V7539', 566986000, 'SG', 'American Bureau of Shipping', 2237,
   15226, 1400, 158940, 156025.00, 76300.00, 21.0, 58400.0, 366.00, 51.20, 15.50,
   2018, 23,  512770, TRUE, b'1',
   'H-class. Deployed on the Far East to North Europe rotation.'),
  ('9433781', 'Elbe Trader', 'V2CX3', 305238000, 'AG', 'RINA', 1129,
   1036, 220, 13760, 9996.00, 4998.00, 18.5, 4200.0, 151.72, 23.40, 8.70,
   2009, 14,  986420, FALSE, b'0',
   'Feeder on the Iberia to Morocco shuttle. Laid up at Leixoes pending sale.');

-- -----------------------------------------------------------------------------
-- voyage
-- -----------------------------------------------------------------------------

INSERT INTO voyage
  (vessel_imo, voyage_number, service_code, etd, eta, atd, status,
   bunker_price_usd_t, slot_allocation_teu, utilisation_pct)
VALUES
  ('9839430', '2026-014W', 'AE7 ', '2026-03-02 18:00:00', '2026-04-03 06:00:00', '2026-03-02 19:40:00', 'Completed',  612.50, 21000, 94.20),
  ('9839430', '2026-015E', 'AE7 ', '2026-04-08 12:00:00', '2026-05-11 08:00:00', '2026-04-08 13:20:00', 'Sailing',    634.00, 21000, 88.70),
  ('9893890', '2026-008W', 'CEM ', '2026-03-14 09:00:00', '2026-04-16 22:00:00', NULL,                  'Announced',  598.75, 20500, NULL),
  ('9839179', '2026-021W', 'FAL1', '2026-02-27 22:30:00', '2026-03-30 05:00:00', '2026-02-28 01:10:00', 'Completed',  605.20, 20000, 91.05),
  ('9967494', '2026-004E', 'NEU2', '2026-04-21 06:00:00', '2026-05-24 18:00:00', NULL,                  'Planned',    640.10, 21500, NULL),
  ('9784271', '2026-031W', 'WAF3', '2026-03-19 14:00:00', '2026-04-09 09:00:00', '2026-03-19 15:05:00', 'Sailing',    621.90, 13000, 76.40),
  ('9784271', '2026-032E', 'WAF3', '2026-04-14 08:00:00', '2026-05-06 20:00:00', NULL,                  'Planned',    629.00, 13000, NULL),
  ('9433781', '2025-107S', 'IBM1', '2025-11-11 07:00:00', '2025-11-16 16:00:00', '2025-11-11 07:45:00', 'Cancelled',  548.30,   900, NULL);

-- -----------------------------------------------------------------------------
-- voyage_leg
-- -----------------------------------------------------------------------------

INSERT INTO voyage_leg
  (vessel_imo, voyage_number, leg_sequence, port_id, eta, etd, ata,
   berth_shift_time, pilot_required, moves_discharged, moves_loaded)
SELECT '9839430', '2026-014W', 1, port_id, '2026-03-02 06:00:00', '2026-03-02 18:00:00', '2026-03-02 05:41:12.482000', NULL,           TRUE, 1840, 2210 FROM port WHERE un_locode='NLRTM'
UNION ALL SELECT '9839430', '2026-014W', 2, port_id, '2026-03-04 08:00:00', '2026-03-05 02:00:00', '2026-03-04 09:12:03.115000', '00:42:18.500', TRUE, 1120, 1640 FROM port WHERE un_locode='BEANR'
UNION ALL SELECT '9839430', '2026-014W', 3, port_id, '2026-03-08 14:00:00', '2026-03-09 04:00:00', '2026-03-08 13:55:47.900000', NULL,           TRUE,  760, 1180 FROM port WHERE un_locode='ESALG'
UNION ALL SELECT '9839430', '2026-014W', 4, port_id, '2026-04-01 20:00:00', '2026-04-03 06:00:00', NULL,                          NULL,           TRUE, 3200,  410 FROM port WHERE un_locode='CNSHA'
UNION ALL SELECT '9839179', '2026-021W', 1, port_id, '2026-02-27 10:00:00', '2026-02-27 22:30:00', '2026-02-27 10:31:55.000000', NULL,           TRUE, 1510, 1980 FROM port WHERE un_locode='DEHAM'
UNION ALL SELECT '9839179', '2026-021W', 2, port_id, '2026-03-01 06:00:00', '2026-03-01 21:00:00', '2026-03-01 05:48:10.250000', '00:15:00.000', TRUE,  940, 1305 FROM port WHERE un_locode='GBFXT'
UNION ALL SELECT '9839179', '2026-021W', 3, port_id, '2026-03-04 12:00:00', '2026-03-05 01:00:00', '2026-03-04 12:22:39.640000', NULL,           TRUE,  680,  920 FROM port WHERE un_locode='MAPTM'
UNION ALL SELECT '9784271', '2026-031W', 1, port_id, '2026-03-19 04:00:00', '2026-03-19 14:00:00', '2026-03-19 04:07:31.000000', NULL,           TRUE,  610,  880 FROM port WHERE un_locode='PTLIS'
UNION ALL SELECT '9784271', '2026-031W', 2, port_id, '2026-03-24 07:00:00', '2026-03-25 03:00:00', '2026-03-24 09:41:02.700000', '01:05:44.250', TRUE,  430,  510 FROM port WHERE un_locode='SNDKR'
UNION ALL SELECT '9784271', '2026-031W', 3, port_id, '2026-03-29 11:00:00', '2026-03-30 08:00:00', NULL,                          NULL,           TRUE,  520,  360 FROM port WHERE un_locode='CIABJ'
UNION ALL SELECT '9784271', '2026-031W', 4, port_id, '2026-04-05 09:00:00', '2026-04-09 09:00:00', NULL,                          NULL,           TRUE,  700,  240 FROM port WHERE un_locode='NGLOS'
UNION ALL SELECT '9433781', '2025-107S', 1, port_id, '2025-11-11 05:00:00', '2025-11-11 07:00:00', '2025-11-11 05:12:00.000000', NULL,          FALSE,   80,  120 FROM port WHERE un_locode='PTLEI'
UNION ALL SELECT '9433781', '2025-107S', 2, port_id, '2025-11-13 18:00:00', '2025-11-14 06:00:00', NULL,                          NULL,          FALSE,   95,   60 FROM port WHERE un_locode='MAPTM';

-- -----------------------------------------------------------------------------
-- customer
-- -----------------------------------------------------------------------------
-- The IP addresses are drawn from the documentation ranges reserved by
-- RFC 5737 and RFC 3849, so nothing here points at a real host.

INSERT INTO customer
  (legal_name, trading_name, vat_number, eori_number, country_code,
   legacy_account_code, portal_uuid, last_login_ipv4, last_login_ipv6,
   credit_limit_eur, outstanding_eur, risk_score, payment_terms_days,
   preferences, notification_flags, is_active, onboarded_on, contact_note)
VALUES
  ('Atlântico Cargas, Lda.', 'Atlântico Cargas', 'PT501234567', 'PT501234567000', 'PT',
   'AS400-0041', '3f2b7c18-9d4a-4e61-b0c7-2a51d8e93b40', '203.0.113.24', '2001:db8:3f2b::c7',
   750000.00, 218430.55, 18.50, 45,
   '{"invoice_format":"UBL 2.1","language":"pt-PT","edi_partner_id":"PTATLCRG","milestone_alerts":["gate-in","loaded","discharged"]}',
   b'00000111', TRUE, '2011-06-14', 'Contacto principal: Sr.ª Engª Matilde Sá Carneiro — extensão 214'),
  ('Iberian Cold Chain S.A.', 'ICC Reefer', 'ES B87654321', 'ESB87654321', 'ES',
   'AS400-0187', '8c1d5e07-4a2f-4c99-8e13-6b7f0a2c4d51', '198.51.100.87', '2001:db8:8c1d::13',
   1200000.00, 640275.10, 24.75, 30,
   '{"invoice_format":"Factura-e 3.2.2","language":"es-ES","temperature_alerts":true,"setpoint_tolerance_c":0.5}',
   b'00001111', TRUE, '2014-02-03', 'Reefer monitoring 24/7 — aviso obligatorio ante cualquier desviación'),
  ('Rheinland Maschinenbau GmbH', NULL, 'DE811234567', 'DE1234567890123', 'DE',
   NULL, 'b47a9f22-0e6d-4b18-9f3a-5c8e14d7a2b6', '203.0.113.190', NULL,
   2500000.00, 0.00, 6.25, 60,
   '{"invoice_format":"ZUGFeRD 2.2","language":"de-DE","project_cargo":true,"out_of_gauge_notice_days":10}',
   b'00000101', TRUE, '2008-09-22', 'Rechnungsanschrift weicht von der Lieferanschrift ab — Größe beachten'),
  ('Sahel Trading SARL', 'Sahel Trading', 'SN0123456789', NULL, 'SN',
   'AS400-0912', 'd0e4b6a3-7c15-4d82-a9e0-3f61b28c7d94', NULL, NULL,
   180000.00, 174900.00, 71.00, 15,
   '{"invoice_format":"PDF","language":"fr-FR","proforma_required":true}',
   b'00000001', TRUE, '2019-11-08', 'Paiement à la commande jusqu''à nouvel ordre'),
  ('Nordic Seafood Exports AS', 'NSE', 'NO987654321MVA', 'NO987654321', 'NO',
   NULL, '5a9c3e81-2b74-4f60-8d15-7e0a94c1f3b2', '198.51.100.12', '2001:db8:5a9c::81',
   900000.00, 112050.80, 12.00, 30,
   '{"invoice_format":"EHF 3.0","language":"nb-NO","catch_certificate":true,"setpoint_tolerance_c":0.2}',
   b'00001101', TRUE, '2016-04-19', 'Fersk laks — kun direkte levering fra Ålesund'),
  ('Companhia de Cafés do Kwanza', 'CafeKwanza', 'AO5417000123', NULL, 'AO',
   'AS400-1288', 'e6f81205-3d47-4a9b-b62c-08d5194ae7f3', NULL, '2001:db8:e6f8::5',
   340000.00, 289640.25, 58.40, 21,
   '{"invoice_format":"PDF","language":"pt-AO","ico_marks_required":true}',
   b'00000011', TRUE, '2021-07-30', 'Sacaria de juta, 60 kg — marcação ICO obrigatória'),
  ('Bright Harbour Trading Pte Ltd', 'Bright Harbour', 'SG198800123M', NULL, 'SG',
   NULL, 'af372c60-8b91-4e05-97d3-1a6b40e2c85f', '203.0.113.77', '2001:db8:af37::2c',
   1500000.00, 43800.00, 9.10, 30,
   '{"invoice_format":"PEPPOL BIS 3.0","language":"en-SG","consolidation":"CFS"}',
   b'00000101', TRUE, '2013-01-11', NULL),
  ('Douro Granite Works, Lda.', NULL, 'PT509876543', NULL, 'PT',
   'AS400-0455', 'c2841b76-5f30-4d1e-8a97-b3e6094f2d18', NULL, NULL,
   120000.00, 0.00, NULL, 30,
   NULL,
   b'00000001', FALSE, '2009-03-05', 'Conta suspensa desde 2024 — pedras ornamentais, carga fora de medida');

-- -----------------------------------------------------------------------------
-- container_type
-- -----------------------------------------------------------------------------

INSERT INTO container_type
  (iso_code, description, length_ft, height_ft_in, tare_weight_kg, max_payload_kg, internal_volume_cbm, is_reefer, is_tank)
VALUES
  ('22G1', 'General purpose, twenty foot',            20, '8''6"',  2200.00, 28280.00, 33.200, FALSE, FALSE),
  ('42G1', 'General purpose, forty foot',             40, '8''6"',  3750.00, 26730.00, 67.700, FALSE, FALSE),
  ('45G1', 'General purpose, forty foot high cube',   40, '9''6"',  3900.00, 28600.00, 76.400, FALSE, FALSE),
  ('22R1', 'Refrigerated, twenty foot',               20, '8''6"',  3000.00, 27480.00, 28.300, TRUE,  FALSE),
  ('45R1', 'Refrigerated, forty foot high cube',      40, '9''6"',  4800.00, 29200.00, 67.300, TRUE,  FALSE),
  ('22T6', 'Tank, twenty foot, hazardous permitted',  20, '8''6"',  3900.00, 26100.00, 26.000, FALSE, TRUE),
  ('42U1', 'Open top, forty foot',                    40, '8''6"',  4300.00, 26180.00, 64.900, FALSE, FALSE),
  ('42P1', 'Flat rack, forty foot',                   40, '8''6"',  5000.00, 39000.00,  0.001, FALSE, FALSE);

-- -----------------------------------------------------------------------------
-- charge_code
-- -----------------------------------------------------------------------------

INSERT INTO charge_code (charge_code, description, is_freight, vat_rate, gl_account) VALUES
  ('OFR',  'Ocean freight',                              TRUE,  0.0000, '4010-OCEAN'),
  ('BAF',  'Bunker adjustment factor',                   TRUE,  0.0000, '4011-BUNKER'),
  ('CAF',  'Currency adjustment factor',                 TRUE,  0.0000, '4012-CURFX'),
  ('THC',  'Terminal handling charge, origin',           FALSE, 0.2300, '4210-THCORG'),
  ('THCD', 'Terminal handling charge, destination',      FALSE, 0.0000, '4211-THCDST'),
  ('ISPS', 'International ship and port facility charge',FALSE, 0.2300, '4220-ISPS'),
  ('DOC',  'Documentation fee',                          FALSE, 0.2300, '4230-DOCFEE'),
  ('CUST', 'Customs clearance',                          FALSE, 0.2300, '4240-CUSTOM'),
  ('REEF', 'Reefer monitoring and plug-in',              FALSE, 0.2300, '4250-REEFER'),
  ('IMO',  'Dangerous goods surcharge',                  FALSE, 0.2300, '4260-IMDG'),
  ('DEM',  'Demurrage',                                  FALSE, 0.2300, '4310-DEMUR'),
  ('OOG',  'Out of gauge surcharge',                     FALSE, 0.2300, '4270-OOG');

-- -----------------------------------------------------------------------------
-- tariff  (SYSTEM VERSIONED)
-- -----------------------------------------------------------------------------

INSERT INTO tariff
  (trade_lane, origin_region, destination_region, equipment_iso_code, basis,
   currency, base_rate, bunker_surcharge, congestion_surcharge, valid_from, valid_until)
VALUES
  ('North Europe – West Africa', 'North Europe', 'West Africa', '42G1', 'Per container', 'USD', 2450.00, 385.00, 120.00, '2026-01-01', '2026-06-30'),
  ('North Europe – West Africa', 'North Europe', 'West Africa', '22G1', 'Per container', 'USD', 1580.00, 240.00,  80.00, '2026-01-01', '2026-06-30'),
  ('Iberia – Morocco',           'Iberia',       'Morocco',     '42G1', 'Per container', 'EUR',  620.00,  95.00,   0.00, '2026-01-01', '2026-12-31'),
  ('Far East – North Europe',    'Far East',     'North Europe','45G1', 'Per container', 'USD', 3180.00, 470.00, 260.00, '2026-02-01', '2026-07-31'),
  ('Far East – North Europe',    'Far East',     'North Europe','45R1', 'Per container', 'USD', 4750.00, 470.00, 260.00, '2026-02-01', '2026-07-31'),
  ('North Europe – Brazil',      'North Europe', 'South America','42G1','Per container', 'USD', 2890.00, 410.00,  95.00, '2026-03-01', '2026-08-31'),
  ('Project cargo, any lane',    'Any',          'Any',         '42P1', 'Per freight tonne','EUR',  48.50,   6.20,   0.00, '2026-01-01', '2026-12-31');

-- -----------------------------------------------------------------------------
-- consignment
-- -----------------------------------------------------------------------------
-- Surrogate keys are resolved through the natural keys rather than assumed, so
-- the file stays correct whatever AUTO_INCREMENT happens to hand out.

SET @cust_atlantico := (SELECT customer_id FROM customer WHERE legal_name = 'Atlântico Cargas, Lda.');
SET @cust_icc       := (SELECT customer_id FROM customer WHERE legal_name = 'Iberian Cold Chain S.A.');
SET @cust_rheinland := (SELECT customer_id FROM customer WHERE legal_name = 'Rheinland Maschinenbau GmbH');
SET @cust_sahel     := (SELECT customer_id FROM customer WHERE legal_name = 'Sahel Trading SARL');
SET @cust_nordic    := (SELECT customer_id FROM customer WHERE legal_name = 'Nordic Seafood Exports AS');
SET @cust_kwanza    := (SELECT customer_id FROM customer WHERE legal_name = 'Companhia de Cafés do Kwanza');
SET @cust_bright    := (SELECT customer_id FROM customer WHERE legal_name = 'Bright Harbour Trading Pte Ltd');
SET @cust_douro     := (SELECT customer_id FROM customer WHERE legal_name = 'Douro Granite Works, Lda.');

INSERT INTO consignment
  (booking_reference, bill_of_lading_no, shipper_id, consignee_id,
   vessel_imo, voyage_number, origin_locode, destination_locode,
   invoice_currency, payment_due_on, hs_code_override, shipper_reference, broker_reference,
   incoterm, service_scope, handling_flags, status,
   declared_value, freight_amount, insurance_rate, gross_weight_kg, volume_cbm,
   notification_flags, is_hazardous,
   marks_and_numbers, goods_description, packing_list, terms_and_conditions, booked_at)
VALUES
  ('BKG-000471204', 'BL26000000001', @cust_atlantico, @cust_sahel,
   '9784271', '2026-031W', 'PTLIS', 'SNDKR',
   'EUR', '2026-04-18', NULL, 'ATL/2026/0412', 'MAERSK-BR-88213',
   'CIF', 'Port to door', 'Stackable,Direct delivery', 'Loaded',
   184500.00, 3420.00, 0.0035, 21480.500, 58.400,
   b'00000111', FALSE,
   'ATL/DKR 1-420 — NO HOOKS',
   'Portland cement in 25 kg paper sacks, palletised and shrink-wrapped, 420 pallets.',
   '420 pallets, 48 sacks each, 1.20 x 0.80 m, stacked two high.',
   'Carriage subject to the carrier bill of lading terms in force on the date of issue. Liability limited to 2 SDR per kilogramme unless value is declared and ad valorem freight paid.',
   '2026-03-04 09:14:22.317000'),

  ('BKG-000471318', 'BL26000000002', @cust_icc, @cust_bright,
   '9839430', '2026-014W', 'NLRTM', 'CNSHA',
   'USD', '2026-04-25', '0303.14', 'ICC-REEF-2026-0331', NULL,
   'CIF', 'Port to port', 'Temperature controlled,Lloyd''s Register survey required', 'In transit',
   612000.00, 19800.00, 0.0042, 96240.000, 268.000,
   b'00001111', FALSE,
   'ICC/SHA REEFER — KEEP AT -21C',
   'Frozen Atlantic salmon fillets, individually quick frozen, packed in waxed cartons of 20 kg.',
   'Four forty-foot high-cube reefers, 1200 cartons each, setpoint -21 C, ventilation closed.',
   'Reefer cargo carried under the carrier standard temperature-controlled terms. The carrier does not guarantee a temperature inside the packaging, only the air delivery temperature at the evaporator.',
   '2026-02-26 15:41:08.902000'),

  ('BKG-000471455', NULL, @cust_rheinland, @cust_kwanza,
   '9839179', '2026-021W', 'DEHAM', 'MAPTM',
   'EUR', '2026-05-02', NULL, 'RMB-PRJ-1147', 'KUEHNE-DE-40021',
   'FCA', 'Door to port', 'Out of gauge,Fragile', 'Booked',
   1480000.00, 27650.00, NULL, 62300.000, 182.500,
   b'00000101', FALSE,
   'RMB 1147/1-3 — CENTRE OF GRAVITY MARKED',
   'Two CNC gantry milling machines, partially dismantled, with tooling and spare parts.',
   'Three flat racks: bed 1 (24.8 t), bed 2 (24.1 t), column assembly and tooling (13.4 t). Lashing plan attached.',
   'Out of gauge cargo accepted subject to survey. The merchant warrants that the declared centre of gravity is accurate.',
   '2026-02-19 11:02:47.554000'),

  ('BKG-000471502', NULL, @cust_nordic, @cust_icc,
   NULL, NULL, 'NOALS', 'SGSIN',
   'EUR', '2026-05-14', NULL, 'NSE-2026-0088', NULL,
   'FOB', 'Port to port', 'Temperature controlled', 'Draft',
   238400.00, 0.00, NULL, 24800.000, 62.000,
   b'00001101', FALSE,
   'NSE/SIN 1-620',
   'Fresh farmed salmon, gutted, head on, in expanded polystyrene boxes with gel ice.',
   'Awaiting equipment allocation; 620 boxes of 40 kg expected.',
   NULL,
   '2026-04-02 08:20:11.004000'),

  ('BKG-000471590', 'BL26000000005', @cust_kwanza, NULL,
   '9784271', '2026-031W', 'NGLOS', 'CIABJ',
   'USD', '2026-04-30', '0901.11', 'CKW-2026-0217', NULL,
   'FOB', 'Port to port', '', 'Discharged',
   96300.00, 2180.00, 0.0030, 19200.000, 41.800,
   b'00000011', FALSE,
   'CKW 217 — ICO 12/0044/0217',
   'Green coffee beans, Robusta, crop 2025, in jute bags of 60 kg.',
   '320 bags, one twenty-foot dry container, kraft paper lined.',
   'Consigned to order. Delivery against surrender of one original bill of lading duly endorsed.',
   '2026-03-01 07:55:39.120000'),

  ('BKG-000471644', 'BL26000000006', @cust_bright, @cust_atlantico,
   '9839430', '2026-015E', 'CNSHA', 'NLRTM',
   'USD', '2026-05-20', NULL, 'BHT/2026/1180', 'DSV-SG-55210',
   'DDP', 'Door to door', 'Fragile,Stackable', 'In transit',
   418700.00, 14920.00, 0.0038, 41560.750, 148.200,
   b'00000101', TRUE,
   'BHT 1180/1-2 — LITHIUM BATTERIES UN3480',
   'Lithium-ion battery modules for stationary storage, packed in accordance with packing instruction P903.',
   'Two forty-foot high cubes, 24 pallets each, state of charge below 30 per cent.',
   'Dangerous goods accepted only against a signed dangerous goods declaration. The merchant indemnifies the carrier against any consequence of a misdeclaration.',
   '2026-04-05 13:37:02.881000'),

  ('BKG-000471701', NULL, @cust_douro, @cust_rheinland,
   NULL, NULL, 'PTLEI', 'MAPTM',
   'EUR', '2026-04-11', NULL, 'DGW-2026-0031', NULL,
   'EXW', 'Port to port', 'Out of gauge', 'Cancelled',
   54800.00, 0.00, NULL, 27400.000, 11.900,
   b'00000001', FALSE,
   'DGW 31',
   'Rough granite blocks, unpolished, from the Pedras Salgadas quarry.',
   NULL,
   NULL,
   '2026-03-22 16:48:55.700000');

-- -----------------------------------------------------------------------------
-- container
-- -----------------------------------------------------------------------------

SET @cns_cement  := (SELECT consignment_id FROM consignment WHERE booking_reference = 'BKG-000471204');
SET @cns_salmon  := (SELECT consignment_id FROM consignment WHERE booking_reference = 'BKG-000471318');
SET @cns_machine := (SELECT consignment_id FROM consignment WHERE booking_reference = 'BKG-000471455');
SET @cns_fresh   := (SELECT consignment_id FROM consignment WHERE booking_reference = 'BKG-000471502');
SET @cns_coffee  := (SELECT consignment_id FROM consignment WHERE booking_reference = 'BKG-000471590');
SET @cns_battery := (SELECT consignment_id FROM consignment WHERE booking_reference = 'BKG-000471644');
SET @cns_granite := (SELECT consignment_id FROM consignment WHERE booking_reference = 'BKG-000471701');

INSERT INTO container
  (equipment_no, consignment_id, iso_code, seal_number, tare_weight_kg, cargo_weight_kg,
   vgm_method, setpoint_celsius, humidity_pct, vent_setting, stow_position, loaded_at)
VALUES
  ('MSCU4471820', @cns_cement,  '42G1', 'PT0448127', 3750.00, 21480.500, 'Method 1 — weighbridge', NULL, NULL, NULL,      '0140884', '2026-03-19 09:12:40.220000'),
  ('MAEU6612094', @cns_salmon,  '45R1', 'NL7712043', 4800.00, 24060.000, 'Method 1 — weighbridge', -21.0,  85, 'Closed',  '0220186', '2026-03-02 11:04:18.640000'),
  ('MAEU6612105', @cns_salmon,  '45R1', 'NL7712044', 4800.00, 24060.000, 'Method 1 — weighbridge', -21.0,  85, 'Closed',  '0220188', '2026-03-02 11:22:57.310000'),
  ('MAEU6612116', @cns_salmon,  '45R1', 'NL7712045', 4800.00, 24060.000, 'Method 2 — calculated',  -21.0,  85, 'Closed',  '0240182', '2026-03-02 12:01:03.775000'),
  ('MAEU6612127', @cns_salmon,  '45R1', 'NL7712046', 4800.00, 24060.000, 'Method 2 — calculated',  -18.0,  85, 'Closed',  '0240184', '2026-03-02 12:35:44.980000'),
  ('HLXU8830471', @cns_machine, '42P1', 'DE9911204', 5000.00, 24800.000, 'Method 1 — weighbridge', NULL, NULL, NULL,      NULL,      NULL),
  ('HLXU8830482', @cns_machine, '42P1', 'DE9911205', 5000.00, 24100.000, 'Method 1 — weighbridge', NULL, NULL, NULL,      NULL,      NULL),
  ('HLXU8830493', @cns_machine, '42P1', 'DE9911206', 5000.00, 13400.000, 'Not yet verified',       NULL, NULL, NULL,      NULL,      NULL),
  ('CMAU3390117', @cns_coffee,  '22G1', 'NG4400871', 2200.00, 19200.000, 'Method 1 — weighbridge', NULL, NULL, NULL,      '0300142', '2026-03-20 06:41:29.500000'),
  ('ONEU7745230', @cns_battery, '45G1', 'CN8820114', 3900.00, 20780.375, 'Method 1 — weighbridge', NULL, NULL, NULL,      '0480902', '2026-04-08 14:52:11.008000'),
  ('ONEU7745241', @cns_battery, '45G1', 'CN8820115', 3900.00, 20780.375, 'Method 1 — weighbridge', NULL, NULL, NULL,      '0480904', '2026-04-08 15:19:36.442000'),
  ('SUDU2201558', @cns_fresh,   '22R1', NULL,        3000.00,     0.000, 'Not yet verified',        2.0,  90, 'Open 25',  NULL,      NULL);

-- -----------------------------------------------------------------------------
-- cargo_item
-- -----------------------------------------------------------------------------

INSERT INTO cargo_item
  (consignment_id, container_id, line_number, hs_code, description, package_kind,
   package_count, gross_weight_kg, net_weight_kg, volume_cbm, imdg_class, un_number,
   origin_country, unit_value)
SELECT @cns_cement, c.container_id, 1, '2523.29',
       'Portland cement, grey, CEM II/B-L 32,5 N, in 25 kg paper sacks on returnable pallets',
       'Pallet', 420, 21480.500, 21000.000, 58.400, 'Not regulated', NULL, 'PT', 439.29
  FROM container c WHERE c.equipment_no = 'MSCU4471820'
UNION ALL
SELECT @cns_salmon, c.container_id, 1, '0303.14',
       'Frozen Atlantic salmon fillets, individually quick frozen, skin on, waxed cartons of 20 kg',
       'Carton', 1200, 24060.000, 24000.000, 67.000, 'Not regulated', NULL, 'NO', 127.50
  FROM container c WHERE c.equipment_no = 'MAEU6612094'
UNION ALL
SELECT @cns_salmon, c.container_id, 2, '0303.14',
       'Frozen Atlantic salmon fillets, individually quick frozen, skin on, waxed cartons of 20 kg',
       'Carton', 1200, 24060.000, 24000.000, 67.000, 'Not regulated', NULL, 'NO', 127.50
  FROM container c WHERE c.equipment_no = 'MAEU6612105'
UNION ALL
SELECT @cns_salmon, c.container_id, 3, '0303.14',
       'Frozen Atlantic salmon fillets, individually quick frozen, skin on, waxed cartons of 20 kg',
       'Carton', 1200, 24060.000, 24000.000, 67.000, 'Not regulated', NULL, 'NO', 127.50
  FROM container c WHERE c.equipment_no = 'MAEU6612116'
UNION ALL
SELECT @cns_salmon, c.container_id, 4, '0303.14',
       'Frozen Atlantic salmon fillets, individually quick frozen, skin on, waxed cartons of 20 kg',
       'Carton', 1200, 24060.000, 24000.000, 67.000, 'Not regulated', NULL, 'NO', 127.50
  FROM container c WHERE c.equipment_no = 'MAEU6612127'
UNION ALL
SELECT @cns_machine, c.container_id, 1, '8459.61',
       'CNC gantry milling machine, bed assembly, partially dismantled, dimensions 12.4 x 3.6 x 3.1 m',
       'Crate', 1, 24800.000, 24300.000, 138.400, 'Not regulated', NULL, 'DE', 615000.00
  FROM container c WHERE c.equipment_no = 'HLXU8830471'
UNION ALL
SELECT @cns_machine, c.container_id, 2, '8459.61',
       'CNC gantry milling machine, column and cross-rail assembly, cradled and braced',
       'Crate', 1, 24100.000, 23600.000, 31.700, 'Not regulated', NULL, 'DE', 590000.00
  FROM container c WHERE c.equipment_no = 'HLXU8830482'
UNION ALL
SELECT @cns_machine, c.container_id, 3, '8466.93',
       'Tooling, spindle spares and hydraulic power pack for the machines on lines 1 and 2',
       'Crate', 14, 13400.000, 12900.000, 12.400, 'Class 3, Flammable liquids', 1263, 'DE', 19642.86
  FROM container c WHERE c.equipment_no = 'HLXU8830493'
UNION ALL
SELECT @cns_coffee, c.container_id, 1, '0901.11',
       'Green coffee beans, Robusta, crop 2025, jute bags of 60 kg, ICO mark 12/0044/0217',
       'Big bag', 320, 19200.000, 19200.000, 41.800, 'Not regulated', NULL, 'AO', 300.94
  FROM container c WHERE c.equipment_no = 'CMAU3390117'
UNION ALL
SELECT @cns_battery, c.container_id, 1, '8507.60',
       'Lithium-ion battery modules for stationary energy storage, state of charge below 30 per cent, UN3480 packing instruction P903',
       'Pallet', 24, 20780.375, 20200.000, 74.100, 'Class 9, Miscellaneous dangerous goods', 3480, 'CN', 8722.92
  FROM container c WHERE c.equipment_no = 'ONEU7745230'
UNION ALL
SELECT @cns_battery, c.container_id, 2, '8507.60',
       'Lithium-ion battery modules for stationary energy storage, state of charge below 30 per cent, UN3480 packing instruction P903',
       'Pallet', 24, 20780.375, 20200.000, 74.100, 'Class 9, Miscellaneous dangerous goods', 3480, 'CN', 8722.92
  FROM container c WHERE c.equipment_no = 'ONEU7745241'
UNION ALL
SELECT @cns_fresh, NULL, 1, '0302.14',
       'Fresh farmed Atlantic salmon, gutted head on, superior grade, expanded polystyrene boxes with gel ice',
       'Carton', 620, 24800.000, 23560.000, 62.000, 'Not regulated', NULL, 'NO', 384.52;

-- -----------------------------------------------------------------------------
-- charge
-- -----------------------------------------------------------------------------

INSERT INTO charge
  (consignment_id, charge_code, quantity, unit_amount, currency, fx_rate_to_eur, payer, invoiced_on, invoice_number)
VALUES
  (@cns_cement,  'OFR',  1.000,  2450.0000, 'EUR', 1.000000, 'Shipper',   '2026-03-20', 'INV-2026-004417'),
  (@cns_cement,  'BAF',  1.000,   385.0000, 'EUR', 1.000000, 'Shipper',   '2026-03-20', 'INV-2026-004417'),
  (@cns_cement,  'THC',  1.000,   215.0000, 'EUR', 1.000000, 'Shipper',   '2026-03-20', 'INV-2026-004417'),
  (@cns_cement,  'DOC',  1.000,    65.0000, 'EUR', 1.000000, 'Shipper',   '2026-03-20', 'INV-2026-004417'),
  (@cns_cement,  'THCD', 1.000,   248.0000, 'EUR', 1.000000, 'Consignee', NULL,          NULL),

  (@cns_salmon,  'OFR',  4.000,  4750.0000, 'USD', 0.921400, 'Shipper',   '2026-03-05', 'INV-2026-004512'),
  (@cns_salmon,  'BAF',  4.000,   470.0000, 'USD', 0.921400, 'Shipper',   '2026-03-05', 'INV-2026-004512'),
  (@cns_salmon,  'REEF', 4.000,   310.0000, 'USD', 0.921400, 'Shipper',   '2026-03-05', 'INV-2026-004512'),
  (@cns_salmon,  'ISPS', 4.000,    32.5000, 'USD', 0.921400, 'Shipper',   '2026-03-05', 'INV-2026-004512'),
  (@cns_salmon,  'CAF',  1.000,   184.6000, 'USD', 0.921400, 'Shipper',   '2026-03-05', 'INV-2026-004512'),

  (@cns_machine, 'OFR',  3.000,  3180.0000, 'EUR', 1.000000, 'Third party', NULL,        NULL),
  (@cns_machine, 'OOG',  3.000,  1450.0000, 'EUR', 1.000000, 'Third party', NULL,        NULL),
  (@cns_machine, 'DOC',  1.000,    65.0000, 'EUR', 1.000000, 'Third party', NULL,        NULL),
  (@cns_machine, 'CUST', 1.000,   180.0000, 'EUR', 1.000000, 'Third party', NULL,        NULL),

  (@cns_coffee,  'OFR',  1.000,  1580.0000, 'USD', 0.921400, 'Shipper',   '2026-03-25', 'INV-2026-004633'),
  (@cns_coffee,  'BAF',  1.000,   240.0000, 'USD', 0.921400, 'Shipper',   '2026-03-25', 'INV-2026-004633'),
  (@cns_coffee,  'DEM',  3.000,   145.0000, 'USD', 0.921400, 'Consignee', NULL,          NULL),

  (@cns_battery, 'OFR',  2.000,  3180.0000, 'USD', 0.921400, 'Shipper',   '2026-04-10', 'INV-2026-004791'),
  (@cns_battery, 'BAF',  2.000,   470.0000, 'USD', 0.921400, 'Shipper',   '2026-04-10', 'INV-2026-004791'),
  (@cns_battery, 'IMO',  2.000,   285.0000, 'USD', 0.921400, 'Shipper',   '2026-04-10', 'INV-2026-004791'),
  (@cns_battery, 'DOC',  1.000,    75.0000, 'USD', 0.921400, 'Shipper',   '2026-04-10', 'INV-2026-004791');

-- -----------------------------------------------------------------------------
-- customs_declaration
-- -----------------------------------------------------------------------------

INSERT INTO customs_declaration
  (consignment_id, clearance_port_id, mrn, procedure_code, regime, status,
   lodged_at, released_at, duty_amount, vat_amount, officer_remarks)
SELECT @cns_cement, p.port_id, '26PT0001234567890A1', '1000', 'Export', 'Released',
       '2026-03-17 08:12:44.000000', '2026-03-17 11:40:02.000000', 0.00, 0.00,
       'Documentary control only. Sacks sampled at the gate, no discrepancy.'
  FROM port p WHERE p.un_locode = 'PTLIS'
UNION ALL
SELECT @cns_salmon, p.port_id, '26NL0009876543210B7', '1000', 'Export', 'Released',
       '2026-02-28 06:55:19.000000', '2026-02-28 07:31:08.000000', 0.00, 0.00,
       'Catch certificate verified against the Norwegian register.'
  FROM port p WHERE p.un_locode = 'NLRTM'
UNION ALL
SELECT @cns_machine, p.port_id, '26DE0004455667788C3', '3151', 'Temporary admission', 'Under control',
       '2026-02-24 14:03:37.000000', NULL, 0.00, 0.00,
       'Physisch kontrolliert — Maschinennummern stimmen mit der Rechnung überein.'
  FROM port p WHERE p.un_locode = 'DEHAM'
UNION ALL
SELECT @cns_coffee, p.port_id, '26NG0002233445566D9', '4000', 'Import', 'Released',
       '2026-03-31 10:22:05.000000', '2026-04-01 09:14:51.000000', 4815.00, 1107.45,
       'Duty assessed at 5 per cent ad valorem. Phytosanitary certificate on file.'
  FROM port p WHERE p.un_locode = 'CIABJ'
UNION ALL
SELECT @cns_battery, p.port_id, '26CN0007788990011E5', '1000', 'Export', 'Lodged',
       '2026-04-06 02:44:12.000000', NULL, 0.00, 0.00,
       NULL
  FROM port p WHERE p.un_locode = 'CNSHA';

-- -----------------------------------------------------------------------------
-- document
-- -----------------------------------------------------------------------------
-- The binary columns hold small, honest byte strings: a real PNG signature, a
-- real PDF header, and a SHA-256 computed over the file name so that the unique
-- index has something distinct to enforce.

INSERT INTO document
  (declaration_id, document_kind, file_name, media_type, byte_size,
   content_sha256, detached_signature, signature_thumbnail, page_preview,
   scanned_original, archive_bundle, uploaded_by, uploaded_at)
SELECT d.declaration_id, 'Bill of lading', 'BL26000000001.pdf', 'application/pdf', 184320,
       UNHEX(SHA2('BL26000000001.pdf', 256)),
       UNHEX('3082019A06092A864886F70D010702A082018B30820187020101'),
       UNHEX('89504E470D0A1A0A0000000D49484452'),
       UNHEX('255044462D312E370A25E2E3CFD30A'),
       NULL, NULL,
       'marta.figueiredo@atlantico-cargas.example', '2026-03-18 16:04:22.180000'
  FROM customs_declaration d WHERE d.mrn = '26PT0001234567890A1'
UNION ALL
SELECT d.declaration_id, 'Phytosanitary certificate', 'NO-PHYTO-2026-00881.pdf', 'application/pdf', 96470,
       UNHEX(SHA2('NO-PHYTO-2026-00881.pdf', 256)),
       NULL,
       NULL,
       UNHEX('255044462D312E340A'),
       UNHEX('FFD8FFE000104A46494600010100000100010000'),
       NULL,
       'kari.johansen@nordic-seafood.example', '2026-02-27 09:41:07.660000'
  FROM customs_declaration d WHERE d.mrn = '26NL0009876543210B7'
UNION ALL
SELECT d.declaration_id, 'Dangerous goods declaration', 'DGD-BKG-000471644.pdf', 'application/pdf', 71204,
       UNHEX(SHA2('DGD-BKG-000471644.pdf', 256)),
       UNHEX('3082014A06092A864886F70D010702A0820139'),
       UNHEX('89504E470D0A1A0A'),
       NULL,
       NULL,
       UNHEX('504B0304140000000800'),
       'wei.lim@bright-harbour.example', '2026-04-06 03:12:58.402000'
  FROM customs_declaration d WHERE d.mrn = '26CN0007788990011E5'
UNION ALL
SELECT NULL, 'Commercial invoice', 'RMB-PRJ-1147-invoice.pdf', 'application/pdf', 45812,
       UNHEX(SHA2('RMB-PRJ-1147-invoice.pdf', 256)),
       NULL, NULL, NULL, NULL, NULL,
       'buchhaltung@rheinland-maschinenbau.example', '2026-02-20 07:29:44.915000'
UNION ALL
SELECT NULL, 'Certificate of origin', 'CKW-217-ICO-origin.pdf', 'application/pdf', 22940,
       UNHEX(SHA2('CKW-217-ICO-origin.pdf', 256)),
       NULL, NULL, NULL, NULL, NULL,
       'exportacao@cafekwanza.example', '2026-03-02 12:55:31.005000';

-- -----------------------------------------------------------------------------
-- legacy_edi_field
-- -----------------------------------------------------------------------------
-- Deliberately hostile identifiers. The values are the real EDIFACT segment
-- mappings the columns were named after.

INSERT INTO legacy_edi_field
  (segment_tag, `back``tick`, `space in name`, `select`, `posição`, `Mixed Case Column`, retired_on)
VALUES
  ('BGM', 'C106/1004', 'Document number',        'DOCNO',  'Cabeçalho', 'Beginning Of Message', NULL),
  ('NAD', 'C082/3039', 'Party identification',   'PARTY',  'Endereço',  'Name And Address',     NULL),
  ('CNI', 'C503/1004', 'Consignment number',     'CNSNO',  'Remessa',   'Consignment Information', NULL),
  ('MEA', 'C174/6314', 'Measurement value',      'MEASV',  'Medição',   'Measurements',         '2023-12-31'),
  ('DGS', 'C205/8351', 'Dangerous goods class',  'DGCLS',  'Perigosas', 'Dangerous Goods',      NULL);

-- =============================================================================
-- Exercising the behaviour, not just the shape
-- =============================================================================
-- Everything above is a static picture. The statements below make the schema
-- move, so that the objects whose whole point is to react have actually
-- reacted by the time the container reports itself ready.

-- The sequence and the OUT parameters of a stored procedure. This is also the
-- only consignment in the fixture whose booking reference came from
-- booking_reference_seq rather than from the pre-migration numbering.
CALL sp_book_consignment(
  @cust_atlantico, @cust_rheinland,
  'PTLIS', 'NLRTM',
  'Cork stoppers, natural, class 1, in polythene-lined cartons.',
  8420.750,
  'ATL/2026/0518',
  @new_consignment_id, @new_booking_reference
);

-- BEFORE UPDATE, which derives the bill of lading number on the transition to
-- Loaded, and AFTER UPDATE, which records the status change.
UPDATE consignment
   SET status = 'Loaded'
 WHERE booking_reference = 'BKG-000471455';

UPDATE consignment
   SET status = 'Booked',
       vessel_imo = '9784271',
       voyage_number = '2026-032E'
 WHERE booking_reference = 'BKG-000471502';

-- BEFORE DELETE, which authorises the removal, and AFTER DELETE, which records
-- it. The cancelled granite booking has no containers, charges or declarations
-- hanging off it, which is what makes it deletable at all: the foreign keys
-- pointing at consignment are RESTRICT and NO ACTION by design.
DELETE FROM consignment WHERE booking_reference = 'BKG-000471701';

-- A superseded tariff, so that the system-versioned table has a history row and
-- not merely the capacity for one.
UPDATE tariff
   SET base_rate = 2620.00,
       bunker_surcharge = 402.00,
       congestion_surcharge = 155.00
 WHERE trade_lane = 'North Europe – West Africa'
   AND equipment_iso_code = '42G1';

-- Refresh the catalogue statistics, so that the row counts and lengths a reader
-- finds in INFORMATION_SCHEMA reflect the data just loaded rather than an empty
-- table.
ANALYZE TABLE
  port, port_facility, vessel, voyage, voyage_leg, customer, consignment,
  container_type, container, cargo_item, charge_code, charge,
  customs_declaration, document, tariff, audit_event, legacy_edi_field;
