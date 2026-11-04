-- =============================================================================
-- tpl catalogue fixture for the `mysql` provider — DDL
-- =============================================================================
--
-- Schema `freight`: a freight-forwarding / maritime logistics operation, the
-- domain of the MariaDB fixture in `scripts/mariadb/`, adapted to MySQL.
--
-- This file has one job: present the `mysql` provider's catalogue reader with
-- every case `specification/mysql-contract.md` states, in a domain realistic
-- enough that the output of a render is worth looking at. The cases are the
-- sixteen items of FR-MYSQL-044; the comment above each object names the items
-- it carries, and README.md maps every item to the objects that realise it.
--
-- Every statement below is accepted, unchanged, by every series of
-- FR-MYSQL-003 (8.4 and 9.7), so a difference observed between two servers is
-- a difference between the servers. Nothing here is version-gated and nothing
-- is conditional: a structure a supported series refuses is absent from this
-- file, and README.md records the refusal.
--
-- Run order is fixed by the Dockerfile, which copies this file as
-- 01-setup.sql and the data as 02-seed.sql.
-- =============================================================================

SET NAMES utf8mb4;

-- -----------------------------------------------------------------------------
-- Schemas
-- -----------------------------------------------------------------------------
-- Three schemas, each with a job of its own:
--
--   freight        the main schema. Declared with an explicit character set
--                  and an explicit collation that is the server default on no
--                  MySQL series: 8.4 and 9.7 default to utf8mb4_0900_ai_ci, so
--                  utf8mb4_unicode_520_ci tells a stated schema collation from
--                  an inherited one. It is also the MariaDB fixture's choice.
--   masterdata     the second schema of FR-MYSQL-044 item 8: the group's shared
--                  reference data. It declares no character set and no
--                  collation of its own (item 16), so it inherits the server's.
--   freight_empty  a database that holds no object, the second side of the
--                  equality FR-MYSQL-042 takes for NFR-PERF-001. It declares
--                  what `freight` declares, so that the two differ in nothing
--                  but the objects they hold.
--
-- masterdata.country is created first: freight.port references it.

DROP DATABASE IF EXISTS freight;
DROP DATABASE IF EXISTS masterdata;
DROP DATABASE IF EXISTS freight_empty;

CREATE DATABASE masterdata;

CREATE DATABASE freight
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_520_ci;

CREATE DATABASE freight_empty
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_520_ci;

-- The table of the second schema that a table of the main schema references
-- (item 8). Its key column states its character set and collation, because a
-- foreign key needs the referencing and the referenced column to agree and the
-- schema around it states neither.
CREATE TABLE masterdata.country (
  iso_alpha2      CHAR(2) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'ISO 3166-1 alpha-2 code',
  iso_alpha3      CHAR(3) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'ISO 3166-1 alpha-3 code',
  iso_numeric     SMALLINT UNSIGNED NOT NULL COMMENT 'ISO 3166-1 numeric code',
  name            VARCHAR(80) NOT NULL COMMENT 'Short name in English, as published by the ISO 3166 maintenance agency',
  PRIMARY KEY (iso_alpha2),
  UNIQUE KEY uq_country_alpha3 (iso_alpha3)
) ENGINE=InnoDB
  COMMENT='Group register of countries, maintained by the data office and shared by every operating company';

USE freight;

-- -----------------------------------------------------------------------------
-- port
-- -----------------------------------------------------------------------------
-- Items 2, 3, 8, 13. Referenced by port_facility, voyage_leg and
-- customs_declaration, and itself references masterdata.country: the outgoing
-- key that crosses a schema boundary (item 8), which the model does not carry,
-- per FR-CAT-057.

CREATE TABLE port (
  port_id             SMALLINT UNSIGNED NOT NULL AUTO_INCREMENT
                      COMMENT 'Surrogate key; UN/LOCODE is the natural key',
  un_locode           CHAR(5) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                      COMMENT 'UN/LOCODE, e.g. PTLIS — two-letter country plus three-letter location',
  name                VARCHAR(120) NOT NULL
                      COMMENT 'Port name as published by the port authority',
  country_code        CHAR(2) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                      COMMENT 'ISO 3166-1 alpha-2, from the group register in masterdata.country',
  local_name          VARCHAR(120) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL
                      COMMENT 'Name in the local language — binary collation, since Ålesund and Alesund are different ports; Japanese names may need 𠮷, outside the BMP',
  timezone_name       VARCHAR(64) CHARACTER SET ascii COLLATE ascii_bin NOT NULL DEFAULT 'UTC'
                      COMMENT 'IANA time zone identifier',
  utc_offset_minutes  SMALLINT NOT NULL DEFAULT 0
                      COMMENT 'Standard-time offset in minutes; negative west of Greenwich',
  berth_count         TINYINT UNSIGNED NOT NULL DEFAULT 1
                      COMMENT 'Number of commercial berths',
  max_draught_m       DECIMAL(5,2) NOT NULL
                      COMMENT 'Maximum permitted draught at the deepest berth, in metres',
  tidal_range_m       FLOAT DEFAULT NULL
                      COMMENT 'Mean spring tidal range, in metres',
  annual_throughput_teu INT UNSIGNED DEFAULT NULL
                      COMMENT 'Container throughput for the last reported calendar year',
  customs_round_clock BIT(1) NOT NULL DEFAULT b'0'
                      COMMENT 'Customs desk staffed 24/7; bit literal default',
  harbour_dues_currency CHAR(3) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT 'EUR'
                      COMMENT 'ISO 4217 code in which harbour dues are invoiced',
  approach_notes      TEXT
                      COMMENT 'Free text from the pilot book',
  PRIMARY KEY (port_id),
  UNIQUE KEY uq_port_un_locode (un_locode),
  KEY idx_port_country_name (country_code, name),
  CONSTRAINT fk_port_country FOREIGN KEY (country_code)
    REFERENCES masterdata.country (iso_alpha2)
    ON DELETE RESTRICT ON UPDATE CASCADE,
  CONSTRAINT chk_port_draught CHECK (max_draught_m > 0)
) ENGINE=InnoDB
  COMMENT='Portos marítimos, respectivos códigos UN/LOCODE e características de acostagem — ver anexo técnico do manual de operações';

-- -----------------------------------------------------------------------------
-- port_facility
-- -----------------------------------------------------------------------------
-- Items 2 and 7: the spatial surface. Every geometry type MySQL implements is
-- present. The berth outline is the column with an SRID (item 2) and carries
-- the SPATIAL index (item 7). It is NOT NULL because every series refuses a
-- SPATIAL index over a nullable column (ERROR 1252), and it has the SRID
-- because over a column without one the index is accepted with warning 3674
-- and never used by the optimizer.

CREATE TABLE port_facility (
  facility_id     INT UNSIGNED NOT NULL AUTO_INCREMENT,
  port_id         SMALLINT UNSIGNED NOT NULL,
  designation     VARCHAR(80) NOT NULL COMMENT 'Terminal or berth designation, e.g. "Alcântara Norte"',
  facility_kind   ENUM('Container terminal','Ro-Ro ramp','Bulk quay','Liquid bulk jetty','Reefer terminal','Lay-by berth')
                  NOT NULL DEFAULT 'Container terminal',
  quay_length_m   DECIMAL(7,2) NOT NULL COMMENT 'Continuous quay length in metres',
  crane_count     TINYINT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'Ship-to-shore gantry cranes',
  gate_opens_at   TIME NOT NULL DEFAULT '06:00:00' COMMENT 'Landside gate opening, local time',
  gate_closes_at  TIME NOT NULL DEFAULT '22:00:00' COMMENT 'Landside gate closing, local time',
  berth_outline   POLYGON NOT NULL SRID 4326 COMMENT 'Berth footprint as surveyed, WGS 84',
  approach_channel LINESTRING DEFAULT NULL COMMENT 'Centreline of the dredged approach',
  anchorage_points MULTIPOINT DEFAULT NULL COMMENT 'Designated waiting anchorages',
  fairway_network MULTILINESTRING DEFAULT NULL COMMENT 'Buoyed fairways serving the facility',
  restricted_areas MULTIPOLYGON DEFAULT NULL COMMENT 'Areas closed to commercial traffic',
  reference_marker POINT DEFAULT NULL COMMENT 'Single published reference position',
  survey_footprint GEOMETRY DEFAULT NULL COMMENT 'Latest survey outline, type not fixed in advance',
  notices_to_mariners GEOMETRYCOLLECTION DEFAULT NULL COMMENT 'Mixed geometry accompanying current notices',
  PRIMARY KEY (facility_id),
  UNIQUE KEY uq_facility_port_designation (port_id, designation),
  SPATIAL KEY sp_facility_berth_outline (berth_outline) COMMENT 'SPATIAL index over the column with an SRID',
  CONSTRAINT fk_facility_port FOREIGN KEY (port_id) REFERENCES port (port_id)
    ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT chk_facility_gate_window CHECK (gate_closes_at > gate_opens_at)
) ENGINE=InnoDB
  COMMENT='Terminais e postos de acostagem, com a respectiva geometria — 港湾施設';

-- -----------------------------------------------------------------------------
-- vessel
-- -----------------------------------------------------------------------------
-- Items 2, 7, 9. Natural primary key: the IMO number is assigned once and never
-- reissued. class_society carries an ENUM member holding an apostrophe, and is
-- defaulted to it. capacity_teu carries the named column-level CHECK
-- constraint; the dimension check is the one declared NOT ENFORCED.

CREATE TABLE vessel (
  imo_number          CHAR(7) CHARACTER SET ascii COLLATE ascii_bin NOT NULL
                      COMMENT 'IMO ship identification number, seven digits, check digit included',
  name                VARCHAR(100) NOT NULL COMMENT 'Registered name; changes on sale',
  call_sign           VARCHAR(10) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  mmsi                INT UNSIGNED DEFAULT NULL COMMENT 'AIS Maritime Mobile Service Identity, nine digits',
  flag_state          CHAR(2) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  class_society       ENUM('Lloyd''s Register','DNV','Bureau Veritas','ClassNK','American Bureau of Shipping','RINA')
                      NOT NULL DEFAULT 'Lloyd''s Register'
                      COMMENT 'Classification society; note the apostrophe in the first member and in the default',
  hull_number         MEDIUMINT DEFAULT NULL COMMENT 'Builder yard number; signed, as some yards used negative rework codes',
  capacity_teu        MEDIUMINT UNSIGNED NOT NULL
                      CONSTRAINT chk_vessel_capacity CHECK (capacity_teu > 0)
                      COMMENT 'Nominal intake in twenty-foot equivalent units',
  reefer_plugs        SMALLINT UNSIGNED NOT NULL DEFAULT 0,
  deadweight_tonnes   INT UNSIGNED NOT NULL,
  gross_tonnage       DECIMAL(10,2) NOT NULL COMMENT 'ITC 1969 gross tonnage',
  net_tonnage         NUMERIC(10,2) DEFAULT NULL COMMENT 'Declared as NUMERIC to observe how the catalogue reports the synonym',
  service_speed_knots FLOAT NOT NULL DEFAULT 18.5,
  ballast_capacity_m3 DOUBLE DEFAULT NULL,
  loa_m               DECIMAL(6,2) NOT NULL COMMENT 'Length overall, metres',
  beam_m              DECIMAL(5,2) NOT NULL,
  design_draught_m    DECIMAL(5,2) NOT NULL,
  year_built          YEAR NOT NULL,
  crew_complement     TINYINT UNSIGNED NOT NULL DEFAULT 22,
  lifetime_distance_nm BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'Cumulative logged distance, nautical miles',
  in_service          BOOLEAN NOT NULL DEFAULT TRUE COMMENT 'Declared BOOLEAN; MySQL stores tinyint(1)',
  scrubber_fitted     BIT(1) NOT NULL DEFAULT b'0',
  particulars_sheet   MEDIUMTEXT COMMENT 'Full ship particulars as supplied by the owner',
  PRIMARY KEY (imo_number),
  UNIQUE KEY uq_vessel_mmsi (mmsi),
  KEY idx_vessel_name_prefix (name(20)) COMMENT 'Prefix index: 20 characters discriminate vessel names well enough',
  CONSTRAINT chk_vessel_dimensions CHECK (loa_m > beam_m AND design_draught_m > 0) NOT ENFORCED
) ENGINE=InnoDB
  COMMENT='Navios porta-contentores em serviço ou fretados';

-- -----------------------------------------------------------------------------
-- voyage
-- -----------------------------------------------------------------------------
-- Items 2, 6, 7, 9. Composite primary key: a voyage number is unique only
-- within a vessel. Carries the descending index, because the operations desk
-- always reads the schedule newest-first, and a functional index on the year
-- of departure, which the annual liner statistics group by. The direction
-- check holds string literals in its clause.

CREATE TABLE voyage (
  vessel_imo      CHAR(7) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  voyage_number   VARCHAR(12) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'Carrier voyage number, e.g. 2026-014W; the last letter is the direction',
  service_code    CHAR(4) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'Liner service the voyage is operated under',
  etd             DATETIME NOT NULL COMMENT 'Estimated time of departure from the first load port',
  eta             DATETIME NOT NULL COMMENT 'Estimated time of arrival at the last discharge port',
  atd             DATETIME DEFAULT NULL COMMENT 'Actual time of departure; NULL until the vessel sails',
  status          ENUM('Planned','Announced','Sailing','Completed','Cancelled') NOT NULL DEFAULT 'Planned',
  bunker_price_usd_t DECIMAL(9,2) DEFAULT NULL COMMENT 'VLSFO price used for the bunker adjustment factor',
  slot_allocation_teu MEDIUMINT UNSIGNED NOT NULL DEFAULT 0,
  utilisation_pct DECIMAL(5,2) DEFAULT NULL,
  schedule_deviation_min INT DEFAULT NULL
                  COMMENT 'Arrival deviation from the long-term schedule, in minutes; negative when early',
  PRIMARY KEY (vessel_imo, voyage_number),
  KEY idx_voyage_etd_desc (etd DESC) COMMENT 'Descending index: the schedule is always read newest-first',
  KEY idx_voyage_service_etd (service_code, etd),
  KEY idx_voyage_departure_year ((YEAR(etd))) COMMENT 'Functional index: the annual liner statistics group by year of departure',
  CONSTRAINT fk_voyage_vessel FOREIGN KEY (vessel_imo) REFERENCES vessel (imo_number)
    ON DELETE CASCADE ON UPDATE RESTRICT,
  CONSTRAINT chk_voyage_window CHECK (eta > etd),
  CONSTRAINT chk_voyage_utilisation CHECK (utilisation_pct IS NULL OR (utilisation_pct >= 0 AND utilisation_pct <= 100)),
  CONSTRAINT chk_voyage_direction CHECK (RIGHT(voyage_number, 1) IN ('N', 'S', 'E', 'W'))
) ENGINE=InnoDB
  COMMENT='Viagens programadas por navio e número de viagem';

-- -----------------------------------------------------------------------------
-- voyage_leg
-- -----------------------------------------------------------------------------
-- Items 2, 5, 6, 8. Three-column primary key, a composite foreign key back to
-- voyage, a virtual generated column with an index on it, and the signed
-- BIGINT of item 2.

CREATE TABLE voyage_leg (
  vessel_imo        CHAR(7) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  voyage_number     VARCHAR(12) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  leg_sequence      TINYINT UNSIGNED NOT NULL COMMENT 'Rotation position, 1 for the first call',
  port_id           SMALLINT UNSIGNED NOT NULL,
  eta               DATETIME NOT NULL,
  etd               DATETIME NOT NULL,
  ata               DATETIME(6) DEFAULT NULL COMMENT 'Actual arrival, microsecond precision as reported by AIS',
  berth_shift_time  TIME(3) DEFAULT NULL COMMENT 'Time spent shifting between berths during the call',
  pilot_required    BOOLEAN NOT NULL DEFAULT TRUE,
  moves_discharged  SMALLINT UNSIGNED NOT NULL DEFAULT 0,
  moves_loaded      SMALLINT UNSIGNED NOT NULL DEFAULT 0,
  moves_total       SMALLINT UNSIGNED AS (moves_discharged + moves_loaded) VIRTUAL
                    COMMENT 'Virtual generated column: total crane moves for the call',
  bunker_delta_kg   BIGINT DEFAULT NULL
                    COMMENT 'Fuel on board at departure minus fuel on board at arrival, kilograms; negative when the call included bunkering',
  PRIMARY KEY (vessel_imo, voyage_number, leg_sequence),
  KEY idx_leg_port_eta (port_id, eta),
  KEY idx_leg_moves_total (moves_total) COMMENT 'Index over a virtual generated column',
  CONSTRAINT fk_leg_voyage FOREIGN KEY (vessel_imo, voyage_number)
    REFERENCES voyage (vessel_imo, voyage_number)
    ON DELETE CASCADE ON UPDATE NO ACTION,
  CONSTRAINT fk_leg_port FOREIGN KEY (port_id) REFERENCES port (port_id)
    ON DELETE RESTRICT ON UPDATE CASCADE,
  CONSTRAINT chk_leg_window CHECK (etd >= eta)
) ENGINE=InnoDB
  COMMENT='Escalas de cada viagem, pela ordem da rotação';

-- -----------------------------------------------------------------------------
-- customer
-- -----------------------------------------------------------------------------
-- Items 2, 3, 4, 5, 7, 8, 9. The densest table of the fixture:
--
--   * the invisible column of item 4 (legacy_account_code), and the expression
--     default that is invisible (crm_sync_token);
--   * a DEFAULT_GENERATED column that also carries `on update` (updated_at);
--   * the JSON column, with a parenthesised expression default holding string
--     literals (item 3), and the multi-valued index over it (item 7);
--   * the UNIQUE functional index of item 7, whose expression holds a string
--     literal;
--   * the self-referencing foreign key of item 8;
--   * the CHECK constraint declared NOT ENFORCED (item 9): the credit desk may
--     book over the limit with a manager's override, so the policy is recorded
--     in the schema and left to the application.
--
-- MySQL has no UUID or INET type: the portal identifier is BINARY(16) in the
-- UUID_TO_BIN form and the last login address is VARBINARY(16) in the
-- INET6_ATON form, which is how a MySQL schema stores both.

CREATE TABLE customer (
  customer_id       INT UNSIGNED NOT NULL AUTO_INCREMENT,
  parent_customer_id INT UNSIGNED DEFAULT NULL
                    COMMENT 'Group parent for consolidated invoicing; NULL for a top-level customer',
  legal_name        VARCHAR(160) NOT NULL COMMENT 'Registered company name as it appears on the bill of lading',
  trading_name      VARCHAR(160) DEFAULT NULL,
  vat_number        VARCHAR(20) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  eori_number       VARCHAR(17) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT NULL
                    COMMENT 'Economic Operators Registration and Identification number',
  country_code      CHAR(2) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  legacy_account_code VARCHAR(24) CHARACTER SET latin1 COLLATE latin1_general_ci DEFAULT NULL INVISIBLE
                    COMMENT 'INVISIBLE: account code from the 1998 AS/400 ledger, kept for reconciliation only',
  crm_sync_token    CHAR(36) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT (UUID()) INVISIBLE
                    COMMENT 'INVISIBLE expression default: the token the CRM synchronisation matches rows on',
  portal_uuid       BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), 1))
                    COMMENT 'Opaque identifier exposed to the customer portal, in the time-ordered UUID_TO_BIN form',
  last_login_ip     VARBINARY(16) DEFAULT NULL
                    COMMENT 'INET6_ATON form: four bytes for an IPv4 address, sixteen for IPv6',
  credit_limit_eur  DECIMAL(14,2) NOT NULL DEFAULT 0.00,
  outstanding_eur   DECIMAL(14,2) NOT NULL DEFAULT 0.00,
  credit_headroom_eur DECIMAL(14,2) AS (credit_limit_eur - outstanding_eur) STORED
                    COMMENT 'Stored generated column: what the sales desk is allowed to book today',
  risk_score        NUMERIC(5,2) DEFAULT NULL COMMENT 'Internal credit risk, 0.00 best to 100.00 worst',
  payment_terms_days SMALLINT UNSIGNED NOT NULL DEFAULT 30,
  preferences       JSON DEFAULT (JSON_OBJECT('language', 'en-GB', 'invoice_format', 'PDF'))
                    COMMENT 'Portal preferences; a native JSON column with an expression default',
  notification_flags BIT(8) NOT NULL DEFAULT b'00000101'
                    COMMENT 'Bit 0 e-mail, bit 1 SMS, bit 2 EDI, bit 3 portal',
  is_active         BOOLEAN NOT NULL DEFAULT TRUE,
  onboarded_on      DATE NOT NULL,
  review_due_on     DATE NOT NULL DEFAULT (CURRENT_DATE + INTERVAL 365 DAY)
                    COMMENT 'Expression default: annual KYC review',
  contact_note      VARCHAR(255) CHARACTER SET utf8mb3 COLLATE utf8mb3_general_ci DEFAULT NULL
                    COMMENT 'Column charset differs from the table default: utf8mb3 from the legacy CRM import',
  created_at        DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  updated_at        TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (customer_id),
  UNIQUE KEY uq_customer_vat (country_code, vat_number),
  UNIQUE KEY uq_customer_vat_normalised ((UPPER(REPLACE(vat_number, ' ', ''))))
    COMMENT 'UNIQUE functional index: one customer per VAT number however it was typed',
  UNIQUE KEY uq_customer_portal_uuid (portal_uuid),
  KEY idx_customer_parent (parent_customer_id),
  KEY idx_customer_legal_name_prefix (legal_name(24)) COMMENT 'Prefix index over the first 24 characters',
  KEY idx_customer_active_review (is_active, review_due_on DESC),
  KEY idx_customer_milestone_alerts ((CAST(preferences->'$.milestone_alerts' AS CHAR(20) ARRAY)))
    COMMENT 'Multi-valued index: which customers subscribe to a given milestone',
  CONSTRAINT fk_customer_parent FOREIGN KEY (parent_customer_id) REFERENCES customer (customer_id)
    ON DELETE NO ACTION ON UPDATE SET NULL,
  CONSTRAINT chk_customer_credit CHECK (credit_limit_eur >= 0 AND outstanding_eur >= 0),
  CONSTRAINT chk_customer_risk CHECK (risk_score IS NULL OR (risk_score >= 0 AND risk_score <= 100)),
  CONSTRAINT chk_customer_credit_policy CHECK (outstanding_eur <= credit_limit_eur) NOT ENFORCED
) ENGINE=InnoDB
  COMMENT='Clientes expedidores e destinatários, com limite de crédito e preferências do portal';

-- -----------------------------------------------------------------------------
-- currency
-- -----------------------------------------------------------------------------
-- Item 6: a table with a NOT NULL unique index and no primary key. InnoDB
-- clusters it on that index, and the catalogue reports the index under its own
-- name rather than as PRIMARY.

CREATE TABLE currency (
  iso_code        CHAR(3) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'ISO 4217 alphabetic code',
  numeric_code    SMALLINT UNSIGNED NOT NULL COMMENT 'ISO 4217 numeric code',
  name            VARCHAR(60) NOT NULL,
  minor_units     TINYINT UNSIGNED NOT NULL DEFAULT 2 COMMENT 'Decimal places of the minor unit',
  UNIQUE KEY uq_currency_iso_code (iso_code),
  KEY idx_currency_numeric_code (numeric_code)
) ENGINE=InnoDB
  COMMENT='Moedas ISO 4217 aceites na facturação';

-- -----------------------------------------------------------------------------
-- reference_counter
-- -----------------------------------------------------------------------------
-- MySQL has no sequence. Booking references are drawn from a counter row
-- instead, with the LAST_INSERT_ID(expr) idiom the MySQL manual gives for
-- emulating one, so that a cancelled booking never reuses a reference a
-- customer has already seen on paper.

CREATE TABLE reference_counter (
  counter_name    VARCHAR(32) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  next_value      BIGINT UNSIGNED NOT NULL COMMENT 'Last value handed out; the next call returns this plus one',
  PRIMARY KEY (counter_name)
) ENGINE=InnoDB
  COMMENT='Contadores de referências, em substituição das sequências que o MySQL não tem';

-- -----------------------------------------------------------------------------
-- consignment
-- -----------------------------------------------------------------------------
-- Items 2, 3, 4, 5, 8, 9. The centre of the model. Holds the default kinds of
-- FR-MYSQL-029 side by side, the SET column with an empty-string default, the
-- whole TEXT size ladder but LONGTEXT, the fractional-second ON UPDATE, the
-- generated column that is invisible, and the foreign key written with neither
-- referential clause (fk_consignment_shipper). The voyage reference is
-- nullable because a consignment is booked before it is assigned to a
-- sailing, which is also what allows that key to be ON DELETE SET NULL.

CREATE TABLE consignment (
  consignment_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  booking_reference VARCHAR(16) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                    COMMENT 'Drawn from reference_counter and formatted by fn_booking_reference',
  bill_of_lading_no VARCHAR(20) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT NULL
                    COMMENT 'Issued only once the cargo is on board',
  shipper_id        INT UNSIGNED NOT NULL,
  consignee_id      INT UNSIGNED DEFAULT NULL
                    COMMENT 'NULL for a "to order" bill of lading, which names no consignee at issue',
  vessel_imo        CHAR(7) CHARACTER SET ascii COLLATE ascii_bin DEFAULT NULL,
  voyage_number     VARCHAR(12) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT NULL,
  origin_locode     CHAR(5) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  destination_locode CHAR(5) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  -- The default kinds, deliberately adjacent:
  invoice_currency  CHAR(3) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT 'EUR'
                    COMMENT 'String literal default',
  payment_due_on    DATE NOT NULL DEFAULT (CURRENT_DATE + INTERVAL 30 DAY)
                    COMMENT 'Parenthesised expression default on a scalar column: standard net-30 terms',
  hs_code_override  VARCHAR(12) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT NULL
                    COMMENT 'Explicit DEFAULT NULL',
  shipper_reference VARCHAR(64) NOT NULL
                    COMMENT 'No DEFAULT clause at all, and NOT NULL: every booking arrives with the customer reference',
  broker_reference  VARCHAR(64)
                    COMMENT 'No DEFAULT clause and nullable, to contrast with the explicit DEFAULT NULL above',
  incoterm          ENUM('EXW','FCA','FAS','FOB','CFR','CIF','CPT','CIP','DAP','DPU','DDP') NOT NULL DEFAULT 'FOB',
  service_scope     ENUM('Port to port','Door to door','Door to port','Port to door') NOT NULL DEFAULT 'Port to port',
  handling_flags    SET('Fragile','Stackable','Temperature controlled','Lloyd''s Register survey required','Out of gauge','Direct delivery')
                    NOT NULL DEFAULT ''
                    COMMENT 'Empty-string default. Note the apostrophe in the fourth member; a SET member may not contain a comma',
  status            ENUM('Draft','Booked','Loaded','In transit','Discharged','Delivered','Cancelled') NOT NULL DEFAULT 'Draft',
  declared_value    DECIMAL(14,2) NOT NULL DEFAULT 0.00,
  freight_amount    DECIMAL(12,2) NOT NULL DEFAULT 0.00,
  insurance_rate    DECIMAL(6,4) DEFAULT NULL COMMENT 'Rate applied to declared_value, e.g. 0.0035',
  insured_amount    DECIMAL(14,2) AS (ROUND(declared_value * COALESCE(insurance_rate, 0), 2)) VIRTUAL INVISIBLE
                    COMMENT 'Virtual generated column, invisible: derived cover, never stored and kept out of SELECT *',
  gross_weight_kg   DECIMAL(12,3) NOT NULL COMMENT 'Sum of the cargo items, kilograms',
  volume_cbm        DECIMAL(10,3) DEFAULT NULL COMMENT 'Cubic metres',
  notification_flags BIT(8) NOT NULL DEFAULT b'00000001',
  is_hazardous      BOOLEAN NOT NULL DEFAULT FALSE,
  marks_and_numbers TINYTEXT COMMENT 'Shipping marks stencilled on the packages',
  goods_description TEXT NOT NULL COMMENT 'Description of goods as it will print on the bill of lading',
  packing_list      MEDIUMTEXT,
  terms_and_conditions TEXT DEFAULT ('Carriage subject to the carrier bill of lading terms in force on the date of issue.')
                    COMMENT 'Parenthesised expression default on a TEXT column, holding a string literal',
  booked_at         DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  updated_at        TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3)
                    COMMENT 'Fractional-second ON UPDATE',
  PRIMARY KEY (consignment_id),
  UNIQUE KEY uq_consignment_booking_reference (booking_reference),
  UNIQUE KEY uq_consignment_bl (bill_of_lading_no),
  KEY idx_consignment_shipper_status (shipper_id, status),
  KEY idx_consignment_voyage (vessel_imo, voyage_number),
  KEY idx_consignment_booked_at_desc (booked_at DESC),
  KEY idx_consignment_consignee (consignee_id),
  CONSTRAINT fk_consignment_shipper FOREIGN KEY (shipper_id) REFERENCES customer (customer_id),
  CONSTRAINT fk_consignment_consignee FOREIGN KEY (consignee_id) REFERENCES customer (customer_id)
    ON DELETE RESTRICT ON UPDATE SET NULL,
  CONSTRAINT fk_consignment_voyage FOREIGN KEY (vessel_imo, voyage_number)
    REFERENCES voyage (vessel_imo, voyage_number)
    ON DELETE SET NULL ON UPDATE CASCADE,
  CONSTRAINT chk_consignment_weight CHECK (gross_weight_kg > 0),
  CONSTRAINT chk_consignment_volume CHECK (volume_cbm IS NULL OR volume_cbm > 0),
  CONSTRAINT chk_consignment_route CHECK (origin_locode <> destination_locode)
) ENGINE=InnoDB
  COMMENT='Expedições reservadas, do rascunho à entrega';

-- -----------------------------------------------------------------------------
-- container_type
-- -----------------------------------------------------------------------------
-- Items 2, 3, 9. ISO 6346 equipment types. Small, stable lookup. Carries the
-- literal default holding an apostrophe, the signed TINYINT of item 2, and
-- the unnamed table-level CHECK constraint of item 9.

CREATE TABLE container_type (
  iso_code        CHAR(4) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'ISO 6346 size-and-type code, e.g. 22G1',
  description     VARCHAR(80) NOT NULL,
  length_ft       TINYINT UNSIGNED NOT NULL,
  height_ft_in    VARCHAR(6) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT '8''6"'
                  COMMENT 'Nominal external height, feet and inches; the default holds an apostrophe',
  tare_weight_kg  DECIMAL(8,2) NOT NULL,
  max_payload_kg  DECIMAL(8,2) NOT NULL,
  internal_volume_cbm DECIMAL(7,3) NOT NULL,
  min_setpoint_celsius TINYINT DEFAULT NULL
                  COMMENT 'Lowest setpoint the reefer unit holds, degrees Celsius; NULL for a dry box',
  is_reefer       BOOLEAN NOT NULL DEFAULT FALSE,
  is_tank         BOOLEAN NOT NULL DEFAULT FALSE,
  PRIMARY KEY (iso_code),
  CHECK (max_payload_kg > tare_weight_kg)
) ENGINE=InnoDB
  COMMENT='Tipos de contentor normalizados ISO 6346';

-- -----------------------------------------------------------------------------
-- container
-- -----------------------------------------------------------------------------
-- Items 5, 7, 8. The stored generated column with an index on it. The type
-- reference is nullable: when an equipment type is withdrawn from the
-- register, the boxes of that type are re-typed by the yard, and until then
-- they carry no type (ON DELETE SET NULL). MariaDB's fixture writes SET DEFAULT
-- here; FR-MYSQL-044 names the four other actions alone, so this one does not.

CREATE TABLE container (
  container_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  equipment_no    CHAR(11) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'ISO 6346 equipment number: four-letter prefix, six digits, one check digit',
  consignment_id  BIGINT UNSIGNED NOT NULL,
  iso_code        CHAR(4) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT '22G1'
                  COMMENT 'Defaults to the standard dry twenty-foot box; NULL once its type is withdrawn',
  seal_number     VARCHAR(15) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT NULL,
  tare_weight_kg  DECIMAL(8,2) NOT NULL,
  cargo_weight_kg DECIMAL(10,3) NOT NULL DEFAULT 0.000,
  vgm_kg          DECIMAL(10,3) AS (tare_weight_kg + cargo_weight_kg) STORED
                  COMMENT 'Stored generated column: SOLAS verified gross mass',
  vgm_method      ENUM('Method 1 — weighbridge','Method 2 — calculated','Not yet verified') NOT NULL DEFAULT 'Not yet verified',
  setpoint_celsius DECIMAL(4,1) DEFAULT NULL COMMENT 'Reefer setpoint; NULL for a dry box',
  humidity_pct    TINYINT UNSIGNED DEFAULT NULL,
  vent_setting    VARCHAR(12) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT NULL,
  stow_position   VARCHAR(8) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT NULL
                  COMMENT 'Bay-row-tier as loaded, e.g. 0340882',
  loaded_at       DATETIME(6) DEFAULT NULL,
  PRIMARY KEY (container_id),
  UNIQUE KEY uq_container_equipment_consignment (equipment_no, consignment_id),
  KEY idx_container_consignment (consignment_id),
  KEY idx_container_iso_code (iso_code),
  KEY idx_container_vgm (vgm_kg) COMMENT 'Index over a stored generated column',
  CONSTRAINT fk_container_consignment FOREIGN KEY (consignment_id) REFERENCES consignment (consignment_id)
    ON DELETE RESTRICT ON UPDATE RESTRICT,
  CONSTRAINT fk_container_type FOREIGN KEY (iso_code) REFERENCES container_type (iso_code)
    ON DELETE SET NULL ON UPDATE RESTRICT,
  CONSTRAINT chk_container_humidity CHECK (humidity_pct IS NULL OR humidity_pct <= 100),
  CONSTRAINT chk_container_cargo_weight CHECK (cargo_weight_kg >= 0)
) ENGINE=InnoDB
  COMMENT='Contentores afectos a cada expedição, com massa bruta verificada (VGM)';

-- -----------------------------------------------------------------------------
-- cargo_item
-- -----------------------------------------------------------------------------
-- Items 2, 5, 7, 9. Carries the FULLTEXT index, the unnamed column-level CHECK
-- constraint on gross_weight_kg, the virtual chargeable-weight column that the
-- rating engine reads, and the stored generated column whose expression holds
-- a string literal (is_dangerous_goods).

CREATE TABLE cargo_item (
  cargo_item_id   BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id  BIGINT UNSIGNED NOT NULL,
  container_id    BIGINT UNSIGNED DEFAULT NULL
                  COMMENT 'NULL while the item is loose cargo awaiting consolidation',
  line_number     SMALLINT UNSIGNED NOT NULL,
  hs_code         VARCHAR(12) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'Harmonised System commodity code',
  description     TEXT NOT NULL COMMENT 'Commercial description; indexed FULLTEXT for the tariff classification desk',
  package_kind    ENUM('Pallet','Crate','Drum','Bale','Carton','Big bag','Roll','Loose') NOT NULL DEFAULT 'Pallet',
  package_count   MEDIUMINT UNSIGNED NOT NULL,
  gross_weight_kg DECIMAL(12,3) NOT NULL CHECK (gross_weight_kg > 0),
  net_weight_kg   DECIMAL(12,3) NOT NULL,
  volume_cbm      DECIMAL(10,3) NOT NULL,
  chargeable_weight_kg DECIMAL(12,3) AS (GREATEST(gross_weight_kg, volume_cbm * 167)) VIRTUAL
                  COMMENT 'Virtual generated column: IATA-style volumetric rule at 167 kg per cubic metre',
  imdg_class      ENUM('Not regulated','Class 3, Flammable liquids','Class 8, Corrosives','Class 9, Miscellaneous dangerous goods')
                  NOT NULL DEFAULT 'Not regulated'
                  COMMENT 'Note the commas inside three of the members',
  is_dangerous_goods BOOLEAN AS (imdg_class <> 'Not regulated') STORED
                  COMMENT 'Stored generated column whose expression holds a string literal',
  un_number       SMALLINT UNSIGNED DEFAULT NULL COMMENT 'UN dangerous goods number, 1 to 3550',
  origin_country  CHAR(2) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  unit_value      DECIMAL(12,2) NOT NULL DEFAULT 0.00,
  line_value      DECIMAL(14,2) AS (ROUND(unit_value * package_count, 2)) STORED
                  COMMENT 'Stored generated column: value declared for this line',
  PRIMARY KEY (cargo_item_id),
  UNIQUE KEY uq_cargo_item_line (consignment_id, line_number),
  KEY idx_cargo_item_container (container_id),
  KEY idx_cargo_item_hs (hs_code),
  KEY idx_cargo_item_chargeable (chargeable_weight_kg) COMMENT 'Index over a virtual generated column',
  FULLTEXT KEY ft_cargo_item_description (description),
  CONSTRAINT fk_cargo_item_container FOREIGN KEY (container_id) REFERENCES container (container_id)
    ON DELETE CASCADE ON UPDATE SET NULL,
  CONSTRAINT chk_cargo_item_net_le_gross CHECK (net_weight_kg <= gross_weight_kg),
  CONSTRAINT chk_cargo_item_un_number CHECK (un_number IS NULL OR (un_number >= 1 AND un_number <= 3550))
) ENGINE=InnoDB
  COMMENT='Linhas de mercadoria de cada expedição';

-- -----------------------------------------------------------------------------
-- tariff
-- -----------------------------------------------------------------------------
-- The rate card. MariaDB's fixture versions it by the system; MySQL has no
-- system-versioned table, so a superseded rate is simply overwritten here.

CREATE TABLE tariff (
  tariff_id       INT UNSIGNED NOT NULL AUTO_INCREMENT,
  trade_lane      VARCHAR(40) NOT NULL COMMENT 'e.g. North Europe – West Africa',
  origin_region   VARCHAR(40) NOT NULL,
  destination_region VARCHAR(40) NOT NULL,
  equipment_iso_code CHAR(4) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  basis           ENUM('Per container','Per freight tonne','Per cubic metre','Per shipment','Per bill of lading') NOT NULL DEFAULT 'Per container',
  currency        CHAR(3) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT 'USD',
  base_rate       DECIMAL(11,2) NOT NULL,
  bunker_surcharge DECIMAL(11,2) NOT NULL DEFAULT 0.00,
  congestion_surcharge DECIMAL(11,2) NOT NULL DEFAULT 0.00,
  all_in_rate     DECIMAL(11,2) AS (base_rate + bunker_surcharge + congestion_surcharge) STORED
                  COMMENT 'Stored generated column: what the customer is quoted',
  valid_from      DATE NOT NULL,
  valid_until     DATE NOT NULL,
  PRIMARY KEY (tariff_id),
  KEY idx_tariff_lane_validity (trade_lane, valid_from DESC),
  CONSTRAINT chk_tariff_validity CHECK (valid_until > valid_from),
  CONSTRAINT chk_tariff_base_rate CHECK (base_rate > 0)
) ENGINE=InnoDB
  COMMENT='Tabela de fretes em vigor';

-- -----------------------------------------------------------------------------
-- charge_code
-- -----------------------------------------------------------------------------

CREATE TABLE charge_code (
  charge_code     VARCHAR(8) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'Industry charge code, e.g. BAF, THC, ISPS',
  description     VARCHAR(100) NOT NULL,
  is_freight      BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Counts towards ocean freight rather than local charges',
  vat_rate        DECIMAL(5,4) NOT NULL DEFAULT 0.0000,
  gl_account      VARCHAR(12) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  PRIMARY KEY (charge_code),
  CONSTRAINT chk_charge_code_vat CHECK (vat_rate >= 0 AND vat_rate < 1)
) ENGINE=InnoDB
  COMMENT='Códigos de encargo utilizados na facturação';

-- -----------------------------------------------------------------------------
-- charge
-- -----------------------------------------------------------------------------
-- Item 7: the invisible index. Finance believes nothing reads charges by
-- invoice number any more; the index is kept but hidden from the optimizer
-- while that belief is tested, which is what an invisible index is for.

CREATE TABLE charge (
  charge_id       BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id  BIGINT UNSIGNED NOT NULL,
  charge_code     VARCHAR(8) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  quantity        DECIMAL(10,3) NOT NULL DEFAULT 1.000,
  unit_amount     DECIMAL(12,4) NOT NULL,
  currency        CHAR(3) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT 'EUR',
  fx_rate_to_eur  DECIMAL(12,6) NOT NULL DEFAULT 1.000000,
  net_amount      DECIMAL(14,2) AS (ROUND(quantity * unit_amount, 2)) STORED
                  COMMENT 'Stored generated column: the line total before tax',
  payer           ENUM('Shipper','Consignee','Third party') NOT NULL DEFAULT 'Shipper',
  invoiced_on     DATE DEFAULT NULL,
  invoice_number  VARCHAR(20) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT NULL,
  PRIMARY KEY (charge_id),
  KEY idx_charge_consignment (consignment_id),
  KEY idx_charge_code (charge_code),
  KEY idx_charge_invoice (invoice_number) INVISIBLE
    COMMENT 'Invisible index: kept, but hidden from the optimizer until finance confirms nothing reads it',
  CONSTRAINT fk_charge_consignment FOREIGN KEY (consignment_id) REFERENCES consignment (consignment_id)
    ON DELETE NO ACTION ON UPDATE CASCADE,
  CONSTRAINT fk_charge_code FOREIGN KEY (charge_code) REFERENCES charge_code (charge_code)
    ON DELETE NO ACTION ON UPDATE RESTRICT,
  CONSTRAINT chk_charge_quantity CHECK (quantity > 0)
) ENGINE=InnoDB
  COMMENT='Encargos aplicados a cada expedição';

-- -----------------------------------------------------------------------------
-- customs_declaration
-- -----------------------------------------------------------------------------

CREATE TABLE customs_declaration (
  declaration_id  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id  BIGINT UNSIGNED NOT NULL,
  clearance_port_id SMALLINT UNSIGNED DEFAULT NULL
                  COMMENT 'NULL once the port record is retired; the declaration itself must survive',
  mrn             VARCHAR(21) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'Movement Reference Number issued by the customs office',
  procedure_code  CHAR(4) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT '4000',
  regime          ENUM('Export','Import','Transit T1','Transit T2','Temporary admission','Re-export') NOT NULL,
  status          ENUM('Draft','Lodged','Under control','Released','Rejected') NOT NULL DEFAULT 'Draft',
  lodged_at       DATETIME(6) DEFAULT NULL,
  released_at     DATETIME(6) DEFAULT NULL,
  duty_amount     DECIMAL(12,2) NOT NULL DEFAULT 0.00,
  vat_amount      DECIMAL(12,2) NOT NULL DEFAULT 0.00,
  total_payable   DECIMAL(12,2) AS (duty_amount + vat_amount) STORED,
  officer_remarks VARCHAR(500) DEFAULT NULL,
  PRIMARY KEY (declaration_id),
  UNIQUE KEY uq_customs_mrn (mrn),
  KEY idx_customs_consignment (consignment_id),
  KEY idx_customs_port_status (clearance_port_id, status),
  CONSTRAINT fk_customs_consignment FOREIGN KEY (consignment_id) REFERENCES consignment (consignment_id)
    ON DELETE RESTRICT ON UPDATE NO ACTION,
  CONSTRAINT fk_customs_port FOREIGN KEY (clearance_port_id) REFERENCES port (port_id)
    ON DELETE SET NULL ON UPDATE SET NULL,
  CONSTRAINT chk_customs_release_order CHECK (released_at IS NULL OR lodged_at IS NULL OR released_at >= lodged_at)
) ENGINE=InnoDB
  COMMENT='Declarações aduaneiras associadas às expedições';

-- -----------------------------------------------------------------------------
-- document
-- -----------------------------------------------------------------------------
-- Item 2: the binary surface — the whole BLOB size ladder plus fixed and
-- variable binary strings.

CREATE TABLE document (
  document_id     BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  declaration_id  BIGINT UNSIGNED DEFAULT NULL
                  COMMENT 'NULL for a document filed against the consignment rather than a declaration',
  document_kind   ENUM('Bill of lading','Commercial invoice','Packing list','Certificate of origin','Phytosanitary certificate','Dangerous goods declaration','Delivery order')
                  NOT NULL,
  file_name       VARCHAR(255) NOT NULL,
  media_type      VARCHAR(120) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT 'application/pdf',
  byte_size       INT UNSIGNED NOT NULL,
  content_sha256  BINARY(32) NOT NULL COMMENT 'Fixed-width binary: SHA-256 of the stored bytes',
  detached_signature VARBINARY(512) DEFAULT NULL COMMENT 'Variable-width binary: CMS detached signature',
  signature_thumbnail TINYBLOB COMMENT 'Rendered signature strip, always small',
  page_preview    BLOB COMMENT 'First-page raster preview',
  scanned_original MEDIUMBLOB COMMENT 'Full colour scan of the paper original',
  archive_bundle  LONGBLOB COMMENT 'Signed archival package retained for the statutory period',
  uploaded_by     VARCHAR(80) NOT NULL,
  uploaded_at     DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  retention_until DATE NOT NULL DEFAULT (CURRENT_DATE + INTERVAL 3650 DAY)
                  COMMENT 'Expression default: ten-year statutory retention',
  PRIMARY KEY (document_id),
  UNIQUE KEY uq_document_sha256 (content_sha256),
  KEY idx_document_declaration (declaration_id),
  KEY idx_document_kind_uploaded (document_kind, uploaded_at DESC),
  CONSTRAINT fk_document_declaration FOREIGN KEY (declaration_id) REFERENCES customs_declaration (declaration_id)
    ON DELETE SET NULL ON UPDATE NO ACTION,
  CONSTRAINT chk_document_byte_size CHECK (byte_size > 0)
) ENGINE=InnoDB
  COMMENT='Documentos digitalizados e assinados que acompanham cada expedição';

-- -----------------------------------------------------------------------------
-- audit_event
-- -----------------------------------------------------------------------------
-- Item 4: the auto-incremental column that is invisible. Written by the
-- triggers on consignment. The key is invisible so that the reporting tools,
-- which read SELECT *, show the business columns only. No foreign key: the
-- trail has to outlive the rows it records, including the deleted ones.
--
-- It is InnoDB, like every table a statement writes together with another:
-- 9.7 runs with enforce_gtid_consistency ON, and refuses a statement that
-- updates a non-transactional table beside a transactional one with
-- ERROR 1785, which a MyISAM trail written by these triggers would be.

CREATE TABLE audit_event (
  audit_event_id  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT INVISIBLE,
  entity_name     VARCHAR(64) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  entity_key      VARCHAR(64) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  action          ENUM('INSERT','UPDATE','DELETE') NOT NULL,
  trigger_timing  ENUM('BEFORE','AFTER') NOT NULL,
  changed_column  VARCHAR(64) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT NULL,
  old_value       VARCHAR(255) DEFAULT NULL,
  new_value       VARCHAR(255) DEFAULT NULL,
  actor           VARCHAR(96) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT 'unknown',
  client_ip       VARBINARY(16) DEFAULT NULL COMMENT 'INET6_ATON form',
  correlation_uuid BINARY(16) DEFAULT NULL COMMENT 'UUID_TO_BIN form',
  observed_at     DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  detail          TEXT,
  PRIMARY KEY (audit_event_id),
  KEY idx_audit_entity (entity_name, entity_key),
  KEY idx_audit_observed (observed_at)
) ENGINE=InnoDB
  COMMENT='Trilho de auditoria escrito pelos gatilhos; sobrevive à remoção das linhas que regista';

-- -----------------------------------------------------------------------------
-- edi_message_staging
-- -----------------------------------------------------------------------------
-- Items 1 and 6: the table of a second engine, and a table with no primary
-- key and no index at all. The EDI gateway bulk-loads raw interchanges here
-- with one autocommitted statement per batch, and the parser drains it; no
-- statement writes it together with another table, so it needs no
-- transactions, and a key would only slow the load. That is the old case for
-- MyISAM, and the one this table keeps. Its payload is the LONGTEXT of the
-- TEXT ladder. The event ev_purge_edi_staging below is what would empty it.

CREATE TABLE edi_message_staging (
  received_at     DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  sender_id       VARCHAR(35) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'UNB interchange sender identification',
  message_type    CHAR(6) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'UNH message type, e.g. IFTMIN, COPRAR, BAPLIE',
  interchange_ref VARCHAR(14) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'UNB interchange control reference',
  payload         LONGTEXT NOT NULL COMMENT 'The interchange exactly as received'
) ENGINE=MyISAM
  COMMENT='Mensagens EDIFACT recebidas e ainda não processadas; sem chave, por desenho';

-- -----------------------------------------------------------------------------
-- vessel_position
-- -----------------------------------------------------------------------------
-- Item 15: the partitioned table, whose partitions the read excludes, per
-- FR-CAT-021. AIS position reports, partitioned by half-year so that old
-- reports are dropped a partition at a time. The vessel is not a foreign key:
-- InnoDB accepts no foreign key on a partitioned table, in either direction.

CREATE TABLE vessel_position (
  vessel_imo      CHAR(7) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  reported_at     DATETIME(3) NOT NULL COMMENT 'AIS report time, UTC',
  latitude        DECIMAL(8,5) NOT NULL,
  longitude       DECIMAL(8,5) NOT NULL,
  speed_over_ground_knots DECIMAL(4,1) DEFAULT NULL,
  course_over_ground_deg  DECIMAL(4,1) DEFAULT NULL,
  navigational_status TINYINT UNSIGNED NOT NULL DEFAULT 15
                  COMMENT 'ITU-R M.1371 navigational status; 15 means not defined',
  PRIMARY KEY (vessel_imo, reported_at)
) ENGINE=InnoDB
  COMMENT='Posições AIS dos navios, particionadas por semestre'
  PARTITION BY RANGE COLUMNS (reported_at) (
    PARTITION p2025h2 VALUES LESS THAN ('2026-01-01'),
    PARTITION p2026h1 VALUES LESS THAN ('2026-07-01'),
    PARTITION p2026h2 VALUES LESS THAN ('2027-01-01'),
    PARTITION pfuture VALUES LESS THAN (MAXVALUE)
  );

-- -----------------------------------------------------------------------------
-- legacy_edi_field
-- -----------------------------------------------------------------------------
-- Items 2, 3 and 14. IDENTIFIER EDGE CASES and the rarer defaults. The column
-- names were carried over verbatim from the EDIFACT mapping table of the
-- forwarding system this schema replaced, which allowed a backtick, an
-- embedded space, a double quote, a reserved word and non-ASCII letters in a
-- field name. Any tool that reads this catalogue must quote and unquote them
-- all; a backtick inside an identifier is written by doubling it.
--
-- The same table carries the defaults no other table needs: the empty string,
-- the four-letter string 'NULL' the old system wrote for an absent value, a
-- binary literal (x'27', the EDIFACT segment terminator, an apostrophe), the
-- parenthesised expression default on a BLOB (the UNA service string advice),
-- and the integer declared ZEROFILL.

CREATE TABLE legacy_edi_field (
  mapping_id            SMALLINT UNSIGNED NOT NULL AUTO_INCREMENT,
  segment_tag           CHAR(3) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                        COMMENT 'EDIFACT segment tag, e.g. BGM, NAD, CNI',
  element_position      SMALLINT(4) UNSIGNED ZEROFILL NOT NULL
                        COMMENT 'Position of the data element in the segment, zero-padded as the 1998 mapping manual prints it',
  qualifier             VARCHAR(3) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT ''
                        COMMENT 'Code qualifier; the empty string where the element takes none',
  null_marker           VARCHAR(8) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT 'NULL'
                        COMMENT 'Literal text the legacy system wrote for an absent value',
  segment_terminator    BINARY(1) NOT NULL DEFAULT x'27'
                        COMMENT 'Binary literal default: the EDIFACT segment terminator, an apostrophe',
  una_service_string    BLOB DEFAULT (x'554E413A2B2E3F2027')
                        COMMENT 'Parenthesised expression default on a BLOB: the UNA service string advice',
  `back``tick`          VARCHAR(48) DEFAULT NULL
                        COMMENT 'Identifier containing an embedded backtick, doubled when quoted',
  `space in name`       VARCHAR(48) DEFAULT NULL
                        COMMENT 'Identifier containing a space',
  `double"quote`        VARCHAR(48) DEFAULT NULL
                        COMMENT 'Identifier containing a double quote',
  `select`              VARCHAR(48) DEFAULT NULL
                        COMMENT 'Identifier that is a reserved word',
  `posição`             VARCHAR(48) DEFAULT NULL
                        COMMENT 'Identifier containing non-ASCII letters',
  `Mixed Case Column`   VARCHAR(48) DEFAULT NULL
                        COMMENT 'Identifier mixing case and spaces',
  retired_on            DATE DEFAULT NULL,
  PRIMARY KEY (mapping_id),
  UNIQUE KEY `uq segment tag` (segment_tag),
  KEY `idx``backtick` (`back``tick`)
) ENGINE=InnoDB
  COMMENT='CASOS-LIMITE DE IDENTIFICADORES — mapeamento EDIFACT herdado; os nomes das colunas são deliberadamente hostis ao parser';

-- -----------------------------------------------------------------------------
-- equipment_survey
-- -----------------------------------------------------------------------------
-- The two cases FR-MYSQL-044 lists apart from its sixteen items, both of which
-- FR-MYSQL-027 and FR-MYSQL-033 carry: a generated invisible primary key, and
-- a foreign key declared ON DELETE SET DEFAULT ON UPDATE SET DEFAULT.
--
-- Equipment interchange surveys, imported nightly from the container depots'
-- own systems. The import tool creates its tables without a primary key, and
-- the operations database requires one on every table, so the table is created
-- with sql_generate_invisible_primary_key ON: the server adds the invisible
-- column my_row_id and the key PRIMARY on it. The setting is this session's
-- and is put back at once, so every other table keeps the server default, OFF.
--
-- The equipment type is the foreign key declared SET DEFAULT for both rules:
-- a survey of a type the register withdraws or re-codes falls back to the
-- standard dry twenty-foot box, as the depots' systems do. Both series accept
-- the clause, report it, and refuse the action at run time (FR-MYSQL-033).

SET SESSION sql_generate_invisible_primary_key = ON;

CREATE TABLE equipment_survey (
  survey_reference  VARCHAR(20) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                    COMMENT 'Equipment interchange receipt number issued by the depot',
  equipment_no      CHAR(11) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                    COMMENT 'ISO 6346 equipment number of the box surveyed',
  iso_code          CHAR(4) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT '22G1'
                    COMMENT 'Equipment type as surveyed; falls back to the standard dry box when its type is withdrawn',
  depot_locode      CHAR(5) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                    COMMENT 'UN/LOCODE of the depot that made the survey',
  surveyed_at       DATETIME NOT NULL,
  damage_codes      SET('Dent','Hole','Bent','Cut','Corrosion','Missing part') DEFAULT NULL
                    COMMENT 'Damage recorded; NULL for a clean box',
  repair_estimate_eur DECIMAL(8,2) NOT NULL DEFAULT 0.00,
  KEY idx_survey_equipment (equipment_no),
  CONSTRAINT fk_survey_container_type FOREIGN KEY (iso_code) REFERENCES container_type (iso_code)
    ON DELETE SET DEFAULT ON UPDATE SET DEFAULT
) ENGINE=InnoDB
  COMMENT='Vistorias de equipamento importadas dos depósitos, criadas sem chave primária';

SET SESSION sql_generate_invisible_primary_key = OFF;

-- =============================================================================
-- The table of the second schema that references the main one
-- =============================================================================
-- Item 8: the incoming key that crosses a schema boundary. The group's master
-- data management links each forwarding customer to its golden record. The
-- model of a `freight` read never sees this key, per FR-CAT-057.

CREATE TABLE masterdata.customer_mdm_link (
  golden_record_id CHAR(12) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                   COMMENT 'Identifier of the golden record in the group MDM hub',
  source_system    VARCHAR(16) NOT NULL DEFAULT 'freight',
  customer_id      INT UNSIGNED NOT NULL,
  linked_at        DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (golden_record_id, source_system),
  UNIQUE KEY uq_mdm_link_customer (customer_id),
  CONSTRAINT fk_mdm_link_customer FOREIGN KEY (customer_id) REFERENCES freight.customer (customer_id)
    ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB
  COMMENT='Ligações entre os registos operacionais e o registo de referência do grupo';

-- =============================================================================
-- Views
-- =============================================================================
-- Item 10: each check option (none, LOCAL, CASCADED), each security type
-- (DEFINER, INVOKER), updatable and not, and each algorithm a view can declare
-- (UNDEFINED, MERGE, TEMPTABLE).

-- MERGE, SQL SECURITY INVOKER, no check option, updatable.
CREATE ALGORITHM = MERGE SQL SECURITY INVOKER VIEW v_port_directory AS
  SELECT
    p.port_id,
    p.un_locode,
    p.name,
    p.country_code,
    p.max_draught_m,
    p.berth_count
  FROM port AS p;

-- UNDEFINED, written out, SQL SECURITY DEFINER, no check option, updatable,
-- with a restricting predicate: a row updated through it may drop out of it,
-- which is precisely the difference from the two below.
CREATE ALGORITHM = UNDEFINED SQL SECURITY DEFINER VIEW v_active_vessel AS
  SELECT
    v.imo_number,
    v.name,
    v.flag_state,
    v.class_society,
    v.capacity_teu,
    v.service_speed_knots,
    v.year_built
  FROM vessel AS v
  WHERE v.in_service = TRUE;

-- WITH LOCAL CHECK OPTION, algorithm left to the server.
CREATE VIEW v_reefer_container AS
  SELECT
    c.container_id,
    c.equipment_no,
    c.consignment_id,
    c.iso_code,
    c.setpoint_celsius,
    c.humidity_pct,
    c.cargo_weight_kg
  FROM container AS c
  WHERE c.setpoint_celsius IS NOT NULL
  WITH LOCAL CHECK OPTION;

-- WITH CASCADED CHECK OPTION, layered over the view above so that the two
-- written check options are observable side by side.
CREATE VIEW v_reefer_container_deep_frozen AS
  SELECT
    r.container_id,
    r.equipment_no,
    r.consignment_id,
    r.setpoint_celsius
  FROM v_reefer_container AS r
  WHERE r.setpoint_celsius <= -18.0
  WITH CASCADED CHECK OPTION;

-- TEMPTABLE: a multi-table join with aggregation, not updatable, and the view
-- a template is most likely to render.
CREATE ALGORITHM = TEMPTABLE VIEW v_consignment_manifest AS
  SELECT
    cs.consignment_id,
    cs.booking_reference,
    cs.bill_of_lading_no,
    cs.status,
    cs.incoterm,
    shipper.legal_name       AS shipper_name,
    consignee.legal_name     AS consignee_name,
    cs.origin_locode,
    origin_port.name         AS origin_port_name,
    cs.destination_locode,
    destination_port.name    AS destination_port_name,
    ve.name                  AS vessel_name,
    ve.imo_number,
    vo.voyage_number,
    vo.etd,
    vo.eta,
    cs.gross_weight_kg,
    cs.volume_cbm,
    COUNT(DISTINCT ct.container_id) AS container_count,
    COALESCE(SUM(ch.net_amount), 0.00) AS charged_total
  FROM consignment AS cs
  INNER JOIN customer AS shipper          ON shipper.customer_id = cs.shipper_id
  LEFT  JOIN customer AS consignee        ON consignee.customer_id = cs.consignee_id
  LEFT  JOIN port     AS origin_port      ON origin_port.un_locode = cs.origin_locode
  LEFT  JOIN port     AS destination_port ON destination_port.un_locode = cs.destination_locode
  LEFT  JOIN voyage   AS vo               ON vo.vessel_imo = cs.vessel_imo
                                         AND vo.voyage_number = cs.voyage_number
  LEFT  JOIN vessel   AS ve               ON ve.imo_number = vo.vessel_imo
  LEFT  JOIN container AS ct              ON ct.consignment_id = cs.consignment_id
  LEFT  JOIN charge    AS ch              ON ch.consignment_id = cs.consignment_id
  GROUP BY
    cs.consignment_id, cs.booking_reference, cs.bill_of_lading_no, cs.status,
    cs.incoterm, shipper.legal_name, consignee.legal_name, cs.origin_locode,
    origin_port.name, cs.destination_locode, destination_port.name,
    ve.name, ve.imo_number, vo.voyage_number, vo.etd, vo.eta,
    cs.gross_weight_kg, cs.volume_cbm;

-- =============================================================================
-- Stored functions
-- =============================================================================
-- Item 11, with the procedures below. Between the seven routines: IN, OUT and
-- INOUT parameters; deterministic and not; each data-access characteristic
-- (NO SQL, CONTAINS SQL, READS SQL DATA, MODIFIES SQL DATA); each security
-- type; and a comment on every one.
--
-- A stored function's parameters are always IN. Every function declares
-- DETERMINISTIC, NO SQL or READS SQL DATA, because binary logging is on by
-- default on every series and a server then refuses a function that declares
-- none of the three, with ERROR 1418; the data-modifying routine is therefore a
-- procedure.

DELIMITER $$

CREATE FUNCTION fn_volumetric_weight_kg(p_volume_cbm DECIMAL(10,3))
  RETURNS DECIMAL(12,3)
  DETERMINISTIC
  NO SQL
  SQL SECURITY DEFINER
  COMMENT 'Volumetric weight at the industry factor of 167 kg per cubic metre'
BEGIN
  IF p_volume_cbm IS NULL OR p_volume_cbm <= 0 THEN
    RETURN 0.000;
  END IF;
  RETURN ROUND(p_volume_cbm * 167, 3);
END$$

CREATE FUNCTION fn_locode_country(p_locode CHAR(5))
  RETURNS CHAR(2)
  DETERMINISTIC
  CONTAINS SQL
  SQL SECURITY INVOKER
  COMMENT 'The ISO 3166-1 alpha-2 country prefix of a UN/LOCODE'
BEGIN
  IF p_locode IS NULL OR CHAR_LENGTH(p_locode) < 2 THEN
    RETURN NULL;
  END IF;
  RETURN UPPER(LEFT(p_locode, 2));
END$$

CREATE FUNCTION fn_is_hazardous(p_consignment_id BIGINT UNSIGNED)
  RETURNS BOOLEAN
  NOT DETERMINISTIC
  READS SQL DATA
  SQL SECURITY DEFINER
  COMMENT 'TRUE when any cargo item on the consignment carries an IMDG class'
BEGIN
  DECLARE v_count INT DEFAULT 0;
  SELECT COUNT(*) INTO v_count
    FROM cargo_item
   WHERE consignment_id = p_consignment_id
     AND is_dangerous_goods;
  RETURN v_count > 0;
END$$

CREATE FUNCTION fn_booking_reference(p_number BIGINT UNSIGNED)
  RETURNS VARCHAR(16)
  DETERMINISTIC
  NO SQL
  SQL SECURITY DEFINER
  COMMENT 'Formats a counter value as a booking reference, e.g. BKG-000480001'
BEGIN
  RETURN CONCAT('BKG-', LPAD(p_number, 9, '0'));
END$$

-- =============================================================================
-- Stored procedures
-- =============================================================================

CREATE PROCEDURE sp_book_consignment(
  IN  p_shipper_id        INT UNSIGNED,
  IN  p_consignee_id      INT UNSIGNED,
  IN  p_origin_locode     CHAR(5),
  IN  p_destination_locode CHAR(5),
  IN  p_goods_description TEXT,
  IN  p_gross_weight_kg   DECIMAL(12,3),
  IN  p_shipper_reference VARCHAR(64),
  OUT p_consignment_id    BIGINT UNSIGNED,
  OUT p_booking_reference VARCHAR(16)
)
  NOT DETERMINISTIC
  MODIFIES SQL DATA
  SQL SECURITY DEFINER
  COMMENT 'Creates a draft consignment and returns its surrogate key and booking reference'
BEGIN
  UPDATE reference_counter
     SET next_value = LAST_INSERT_ID(next_value + 1)
   WHERE counter_name = 'booking_reference';
  SET p_booking_reference = fn_booking_reference(LAST_INSERT_ID());
  INSERT INTO consignment (
    booking_reference, shipper_id, consignee_id,
    origin_locode, destination_locode,
    shipper_reference, goods_description, gross_weight_kg, status
  ) VALUES (
    p_booking_reference, p_shipper_id, p_consignee_id,
    p_origin_locode, p_destination_locode,
    p_shipper_reference, p_goods_description, p_gross_weight_kg, 'Draft'
  );
  SET p_consignment_id = LAST_INSERT_ID();
END$$

CREATE PROCEDURE sp_recalculate_freight(
  IN    p_consignment_id BIGINT UNSIGNED,
  INOUT p_total_eur      DECIMAL(14,2),
  OUT   p_charge_lines   SMALLINT UNSIGNED
)
  NOT DETERMINISTIC
  READS SQL DATA
  SQL SECURITY INVOKER
  COMMENT 'Adds the consignment charges, converted to euro, onto the running total supplied by the caller'
BEGIN
  DECLARE v_sum DECIMAL(14,2) DEFAULT 0.00;
  DECLARE v_lines SMALLINT UNSIGNED DEFAULT 0;
  SELECT COALESCE(SUM(ROUND(net_amount * fx_rate_to_eur, 2)), 0.00), COUNT(*)
    INTO v_sum, v_lines
    FROM charge
   WHERE consignment_id = p_consignment_id;
  SET p_total_eur = COALESCE(p_total_eur, 0.00) + v_sum;
  SET p_charge_lines = v_lines;
END$$

CREATE PROCEDURE sp_port_call_summary(
  IN    p_vessel_imo  CHAR(7),
  INOUT p_window_days SMALLINT UNSIGNED,
  OUT   p_call_count  INT UNSIGNED
)
  NOT DETERMINISTIC
  READS SQL DATA
  SQL SECURITY DEFINER
  COMMENT 'Counts a vessel''s port calls in a rolling window; clamps and returns the window actually used'
BEGIN
  IF p_window_days IS NULL OR p_window_days = 0 THEN
    SET p_window_days = 90;
  ELSEIF p_window_days > 365 THEN
    SET p_window_days = 365;
  END IF;
  SELECT COUNT(*) INTO p_call_count
    FROM voyage_leg
   WHERE vessel_imo = p_vessel_imo
     AND eta >= (CURRENT_DATE - INTERVAL p_window_days DAY);
END$$

-- =============================================================================
-- Triggers
-- =============================================================================
-- Item 12: seven triggers on consignment — BEFORE and AFTER for each of
-- INSERT, UPDATE and DELETE, and a second BEFORE INSERT ordered after the
-- first with FOLLOWS, so that two triggers share one event and one timing.

CREATE TRIGGER trg_consignment_before_insert
BEFORE INSERT ON consignment
FOR EACH ROW
BEGIN
  SET NEW.booking_reference = UPPER(TRIM(NEW.booking_reference));
  SET NEW.origin_locode = UPPER(NEW.origin_locode);
  SET NEW.destination_locode = UPPER(NEW.destination_locode);
  IF NEW.shipper_reference IS NOT NULL THEN
    SET NEW.shipper_reference = TRIM(NEW.shipper_reference);
  END IF;
END$$

CREATE TRIGGER trg_consignment_before_insert_currency
BEFORE INSERT ON consignment
FOR EACH ROW
FOLLOWS trg_consignment_before_insert
BEGIN
  IF NOT EXISTS (SELECT 1 FROM currency WHERE iso_code = NEW.invoice_currency) THEN
    SIGNAL SQLSTATE '45000'
      SET MESSAGE_TEXT = 'The invoice currency is not an accepted ISO 4217 currency';
  END IF;
END$$

CREATE TRIGGER trg_consignment_after_insert
AFTER INSERT ON consignment
FOR EACH ROW
BEGIN
  INSERT INTO audit_event (entity_name, entity_key, action, trigger_timing, new_value, actor, detail)
  VALUES ('consignment', NEW.booking_reference, 'INSERT', 'AFTER', NEW.status, CURRENT_USER(),
          CONCAT('Booked ', NEW.origin_locode, ' to ', NEW.destination_locode));
END$$

CREATE TRIGGER trg_consignment_before_update
BEFORE UPDATE ON consignment
FOR EACH ROW
BEGIN
  IF OLD.status = 'Delivered' AND NEW.status <> 'Delivered' THEN
    SIGNAL SQLSTATE '45000'
      SET MESSAGE_TEXT = 'A delivered consignment cannot be moved back to an earlier status';
  END IF;
  IF NEW.status = 'Loaded' AND NEW.bill_of_lading_no IS NULL THEN
    SET NEW.bill_of_lading_no = CONCAT('BL', DATE_FORMAT(CURRENT_DATE, '%y'), LPAD(NEW.consignment_id, 9, '0'));
  END IF;
END$$

CREATE TRIGGER trg_consignment_after_update
AFTER UPDATE ON consignment
FOR EACH ROW
BEGIN
  IF NOT (OLD.status <=> NEW.status) THEN
    INSERT INTO audit_event (entity_name, entity_key, action, trigger_timing, changed_column, old_value, new_value, actor)
    VALUES ('consignment', NEW.booking_reference, 'UPDATE', 'AFTER', 'status', OLD.status, NEW.status, CURRENT_USER());
  END IF;
END$$

CREATE TRIGGER trg_consignment_before_delete
BEFORE DELETE ON consignment
FOR EACH ROW
BEGIN
  IF OLD.status NOT IN ('Draft', 'Cancelled') THEN
    SIGNAL SQLSTATE '45000'
      SET MESSAGE_TEXT = 'Only a draft or cancelled consignment may be deleted';
  END IF;
  INSERT INTO audit_event (entity_name, entity_key, action, trigger_timing, old_value, actor, detail)
  VALUES ('consignment', OLD.booking_reference, 'DELETE', 'BEFORE', OLD.status, CURRENT_USER(),
          'Deletion authorised');
END$$

CREATE TRIGGER trg_consignment_after_delete
AFTER DELETE ON consignment
FOR EACH ROW
BEGIN
  INSERT INTO audit_event (entity_name, entity_key, action, trigger_timing, old_value, actor, detail)
  VALUES ('consignment', OLD.booking_reference, 'DELETE', 'AFTER', OLD.status, CURRENT_USER(),
          CONCAT('Removed; declared value was ', OLD.declared_value));
END$$

-- =============================================================================
-- Scheduled event
-- =============================================================================
-- Item 15: the scheduled event, which the read excludes, per FR-CAT-019. It is
-- created DISABLE: in production the DBA enables the nightly purge, and in the
-- fixture the data must not move under a test.

CREATE EVENT ev_purge_edi_staging
  ON SCHEDULE EVERY 1 DAY STARTS '2026-01-01 03:00:00'
  ON COMPLETION PRESERVE
  DISABLE
  COMMENT 'Nightly purge of staged EDI interchanges older than seven days'
  DO
  BEGIN
    DELETE FROM edi_message_staging WHERE received_at < NOW() - INTERVAL 7 DAY;
  END$$

DELIMITER ;

-- =============================================================================
-- Users
-- =============================================================================
-- Item 16: the reduced-grant reader, holding SELECT and EXECUTE on the main
-- schema and nothing more. root, created by the entrypoint from
-- MYSQL_ROOT_PASSWORD and reachable from any host, is the privileged reader.
--
-- No authentication plugin is named: the account takes the server's default,
-- which is the method FR-MYSQL-022 requires `tpl` to authenticate with, and
-- naming it here would hide which method that default is.
--
-- tpl_reader can list the tables and read their rows, and it can call the
-- routines, so they appear to it in the catalogue. What it cannot do is read a
-- definition it holds no privilege for. That is the point of the account — it
-- makes an incomplete read reproducible without having to break anything.

CREATE USER 'tpl_reader'@'%' IDENTIFIED BY 'tpl-reader-pw';
GRANT SELECT, EXECUTE ON freight.* TO 'tpl_reader'@'%';
