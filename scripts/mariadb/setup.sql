-- =============================================================================
-- tpl catalogue fixture — DDL
-- =============================================================================
--
-- Schema `freight`: a freight-forwarding / maritime logistics operation.
--
-- This file has one job: present `tpl`'s catalogue reader with every construct
-- it claims to read, in a domain realistic enough that the output of a render
-- is worth looking at. Every construct below is accepted, unchanged, by all
-- four supported MariaDB series (10.11, 11.4, 11.8, 12.3), so a difference
-- observed between two servers is a difference between the servers.
--
-- Nothing here is version-gated and nothing is conditional. A feature that one
-- supported series rejects is absent from this file; see README.md for the list
-- and the reason in each case.
--
-- Run order is fixed by the Dockerfile, which copies this file as
-- 01-setup.sql and the data as 02-seed.sql.
-- =============================================================================

SET NAMES utf8mb4;

-- -----------------------------------------------------------------------------
-- Schema
-- -----------------------------------------------------------------------------
-- Declared with an explicit character set and an explicit collation. The
-- collation is deliberately not the server default on any supported series, so
-- that a reader can tell a stated schema collation from an inherited one.
--
-- That default is not the same everywhere, which is the reason for the choice:
-- the official images run utf8mb4_general_ci on 10.11 and utf8mb4_uca1400_ai_ci
-- on 11.4, 11.8 and 12.3. Naming either of those would make the schema
-- collation indistinguishable from inheritance on some of the four servers.
-- utf8mb4_unicode_520_ci is available on all four and is the default on none.

DROP DATABASE IF EXISTS freight;
CREATE DATABASE freight
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_520_ci;
USE freight;

-- -----------------------------------------------------------------------------
-- Sequence
-- -----------------------------------------------------------------------------
-- Booking references are drawn from a sequence rather than from AUTO_INCREMENT,
-- because a cancelled booking must not silently reuse a reference a customer has
-- already seen on paper.

CREATE SEQUENCE booking_reference_seq
  START WITH 480001
  INCREMENT BY 1
  MINVALUE 480001
  MAXVALUE 999999999
  CACHE 20
  NOCYCLE;

-- -----------------------------------------------------------------------------
-- port
-- -----------------------------------------------------------------------------
-- Referenced by port_facility, voyage_leg and customs_declaration: three
-- distinct tables pointing at the same parent.

CREATE TABLE port (
  port_id             SMALLINT UNSIGNED NOT NULL AUTO_INCREMENT
                      COMMENT 'Surrogate key; UN/LOCODE is the natural key',
  un_locode           CHAR(5) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                      COMMENT 'UN/LOCODE, e.g. PTLIS — two-letter country plus three-letter location',
  name                VARCHAR(120) NOT NULL
                      COMMENT 'Port name as published by the port authority',
  country_code        CHAR(2) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                      COMMENT 'ISO 3166-1 alpha-2; ASCII by definition, so stored as ASCII',
  local_name          VARCHAR(120) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL
                      COMMENT 'Name in the local language — binary collation, since Ålesund and Alesund are different ports',
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
                      COMMENT 'Customs desk staffed 24/7',
  harbour_dues_currency CHAR(3) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT 'EUR'
                      COMMENT 'ISO 4217 code in which harbour dues are invoiced',
  approach_notes      TEXT DEFAULT NULL
                      COMMENT 'Free text from the pilot book',
  PRIMARY KEY (port_id),
  UNIQUE KEY uq_port_un_locode (un_locode),
  KEY idx_port_country_name (country_code, name),
  CONSTRAINT chk_port_draught CHECK (max_draught_m > 0)
) ENGINE=InnoDB
  COMMENT='Portos marítimos, respectivos códigos UN/LOCODE e características de acostagem — ver anexo técnico do manual de operações';

-- -----------------------------------------------------------------------------
-- port_facility
-- -----------------------------------------------------------------------------
-- Carries the spatial surface. Every OGC geometry type MariaDB implements is
-- present, and the berth outline is indexed spatially, which is why it is
-- NOT NULL: InnoDB refuses a SPATIAL index over a nullable column.

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
  berth_outline   POLYGON NOT NULL COMMENT 'Berth footprint as surveyed',
  approach_channel LINESTRING DEFAULT NULL COMMENT 'Centreline of the dredged approach',
  anchorage_points MULTIPOINT DEFAULT NULL COMMENT 'Designated waiting anchorages',
  fairway_network MULTILINESTRING DEFAULT NULL COMMENT 'Buoyed fairways serving the facility',
  restricted_areas MULTIPOLYGON DEFAULT NULL COMMENT 'Areas closed to commercial traffic',
  reference_marker POINT DEFAULT NULL COMMENT 'Single published reference position',
  survey_footprint GEOMETRY DEFAULT NULL COMMENT 'Latest survey outline, type not fixed in advance',
  notices_to_mariners GEOMETRYCOLLECTION DEFAULT NULL COMMENT 'Mixed geometry accompanying current notices',
  PRIMARY KEY (facility_id),
  UNIQUE KEY uq_facility_port_designation (port_id, designation),
  SPATIAL KEY sp_facility_berth_outline (berth_outline),
  CONSTRAINT fk_facility_port FOREIGN KEY (port_id) REFERENCES port (port_id)
    ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT chk_facility_gate_window CHECK (gate_closes_at > gate_opens_at)
) ENGINE=InnoDB
  COMMENT='Terminais e postos de acostagem, com a respectiva geometria — 港湾施設';

-- -----------------------------------------------------------------------------
-- vessel
-- -----------------------------------------------------------------------------
-- Natural primary key: the IMO number is assigned once and never reissued.
-- class_society carries an ENUM member containing a single quote.

CREATE TABLE vessel (
  imo_number          CHAR(7) CHARACTER SET ascii COLLATE ascii_bin NOT NULL
                      COMMENT 'IMO ship identification number, seven digits, check digit included',
  name                VARCHAR(100) NOT NULL COMMENT 'Registered name; changes on sale',
  call_sign           VARCHAR(10) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  mmsi                INT UNSIGNED DEFAULT NULL COMMENT 'AIS Maritime Mobile Service Identity, nine digits',
  flag_state          CHAR(2) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  class_society       ENUM('Lloyd''s Register','DNV','Bureau Veritas','ClassNK','American Bureau of Shipping','RINA')
                      NOT NULL DEFAULT 'Lloyd''s Register'
                      COMMENT 'Classification society; note the apostrophe in the first member',
  hull_number         MEDIUMINT DEFAULT NULL COMMENT 'Builder yard number; signed, as some yards used negative rework codes',
  capacity_teu        MEDIUMINT UNSIGNED NOT NULL COMMENT 'Nominal intake in twenty-foot equivalent units',
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
  in_service          BOOLEAN NOT NULL DEFAULT TRUE COMMENT 'Declared BOOLEAN; MariaDB stores TINYINT(1)',
  scrubber_fitted     BIT(1) NOT NULL DEFAULT b'0',
  particulars_sheet   MEDIUMTEXT DEFAULT NULL COMMENT 'Full ship particulars as supplied by the owner',
  PRIMARY KEY (imo_number),
  UNIQUE KEY uq_vessel_mmsi (mmsi),
  KEY idx_vessel_name_prefix (name(20)) COMMENT 'Prefix index: 20 characters discriminate vessel names well enough',
  CONSTRAINT chk_vessel_dimensions CHECK (loa_m > beam_m AND design_draught_m > 0)
) ENGINE=InnoDB
  COMMENT='Navios porta-contentores em serviço ou fretados';

-- -----------------------------------------------------------------------------
-- voyage
-- -----------------------------------------------------------------------------
-- Composite primary key: a voyage number is unique only within a vessel.
-- Carries the descending index, because the operations desk always reads the
-- schedule newest-first.

CREATE TABLE voyage (
  vessel_imo      CHAR(7) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  voyage_number   VARCHAR(12) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'Carrier voyage number, e.g. 2026-014W; W and E denote direction',
  service_code    CHAR(4) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'Liner service the voyage is operated under',
  etd             DATETIME NOT NULL COMMENT 'Estimated time of departure from the first load port',
  eta             DATETIME NOT NULL COMMENT 'Estimated time of arrival at the last discharge port',
  atd             DATETIME DEFAULT NULL COMMENT 'Actual time of departure; NULL until the vessel sails',
  status          ENUM('Planned','Announced','Sailing','Completed','Cancelled') NOT NULL DEFAULT 'Planned',
  bunker_price_usd_t DECIMAL(9,2) DEFAULT NULL COMMENT 'VLSFO price used for the bunker adjustment factor',
  slot_allocation_teu MEDIUMINT UNSIGNED NOT NULL DEFAULT 0,
  utilisation_pct DECIMAL(5,2) DEFAULT NULL,
  PRIMARY KEY (vessel_imo, voyage_number),
  KEY idx_voyage_etd_desc (etd DESC) COMMENT 'Descending index: the schedule is always read newest-first',
  KEY idx_voyage_service_etd (service_code, etd),
  CONSTRAINT fk_voyage_vessel FOREIGN KEY (vessel_imo) REFERENCES vessel (imo_number)
    ON DELETE CASCADE ON UPDATE RESTRICT,
  CONSTRAINT chk_voyage_window CHECK (eta > etd),
  CONSTRAINT chk_voyage_utilisation CHECK (utilisation_pct IS NULL OR (utilisation_pct >= 0 AND utilisation_pct <= 100))
) ENGINE=InnoDB
  COMMENT='Viagens programadas por navio e número de viagem';

-- -----------------------------------------------------------------------------
-- voyage_leg
-- -----------------------------------------------------------------------------
-- Three-column primary key, and a composite foreign key back to voyage.

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
-- Holds the INVISIBLE column, the UUID and IP types, and the JSON document.

CREATE TABLE customer (
  customer_id       INT UNSIGNED NOT NULL AUTO_INCREMENT,
  legal_name        VARCHAR(160) NOT NULL COMMENT 'Registered company name as it appears on the bill of lading',
  trading_name      VARCHAR(160) DEFAULT NULL,
  vat_number        VARCHAR(20) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  eori_number       VARCHAR(17) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT NULL
                    COMMENT 'Economic Operators Registration and Identification number',
  country_code      CHAR(2) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  legacy_account_code VARCHAR(24) CHARACTER SET latin1 COLLATE latin1_general_ci DEFAULT NULL INVISIBLE
                    COMMENT 'INVISIBLE: account code from the 1998 AS/400 ledger, kept for reconciliation only',
  portal_uuid       UUID NOT NULL COMMENT 'Opaque identifier exposed to the customer portal',
  last_login_ipv4   INET4 DEFAULT NULL,
  last_login_ipv6   INET6 DEFAULT NULL,
  credit_limit_eur  DECIMAL(14,2) NOT NULL DEFAULT 0.00,
  outstanding_eur   DECIMAL(14,2) NOT NULL DEFAULT 0.00,
  credit_headroom_eur DECIMAL(14,2) AS (credit_limit_eur - outstanding_eur) STORED
                    COMMENT 'Stored generated column: what the sales desk is allowed to book today',
  risk_score        NUMERIC(5,2) DEFAULT NULL COMMENT 'Internal credit risk, 0.00 best to 100.00 worst',
  payment_terms_days SMALLINT UNSIGNED NOT NULL DEFAULT 30,
  preferences       JSON DEFAULT NULL COMMENT 'Portal preferences; MariaDB implements JSON as LONGTEXT plus a json_valid CHECK',
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
  UNIQUE KEY uq_customer_portal_uuid (portal_uuid),
  KEY idx_customer_legal_name_prefix (legal_name(24)) COMMENT 'Prefix index over the first 24 characters',
  KEY idx_customer_active_review (is_active, review_due_on DESC),
  CONSTRAINT chk_customer_credit CHECK (credit_limit_eur >= 0 AND outstanding_eur >= 0),
  CONSTRAINT chk_customer_risk CHECK (risk_score IS NULL OR (risk_score >= 0 AND risk_score <= 100))
) ENGINE=InnoDB
  COMMENT='Clientes expedidores e destinatários, com limite de crédito e preferências do portal';

-- -----------------------------------------------------------------------------
-- consignment
-- -----------------------------------------------------------------------------
-- The centre of the model. Holds the four default kinds side by side, the SET
-- column, and the whole TEXT size ladder. The voyage reference is nullable
-- because a consignment is booked before it is assigned to a sailing, which is
-- also what allows the foreign key to be ON DELETE SET NULL.

CREATE TABLE consignment (
  consignment_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  booking_reference VARCHAR(16) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                    COMMENT 'Drawn from booking_reference_seq, prefixed BKG-',
  bill_of_lading_no VARCHAR(20) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT NULL
                    COMMENT 'Issued only once the cargo is on board',
  shipper_id        INT UNSIGNED NOT NULL,
  consignee_id      INT UNSIGNED DEFAULT NULL
                    COMMENT 'NULL for a "to order" bill of lading, which names no consignee at issue',
  vessel_imo        CHAR(7) CHARACTER SET ascii COLLATE ascii_bin DEFAULT NULL,
  voyage_number     VARCHAR(12) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT NULL,
  origin_locode     CHAR(5) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  destination_locode CHAR(5) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  -- Four default kinds, deliberately adjacent:
  invoice_currency  CHAR(3) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT 'EUR'
                    COMMENT 'Literal default',
  payment_due_on    DATE NOT NULL DEFAULT (CURRENT_DATE + INTERVAL 30 DAY)
                    COMMENT 'Expression default: standard net-30 terms',
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
                    COMMENT 'Note the apostrophe in the fourth member; a SET member may not contain a comma',
  status            ENUM('Draft','Booked','Loaded','In transit','Discharged','Delivered','Cancelled') NOT NULL DEFAULT 'Draft',
  declared_value    DECIMAL(14,2) NOT NULL DEFAULT 0.00,
  freight_amount    DECIMAL(12,2) NOT NULL DEFAULT 0.00,
  insurance_rate    DECIMAL(6,4) DEFAULT NULL COMMENT 'Rate applied to declared_value, e.g. 0.0035',
  insured_amount    DECIMAL(14,2) AS (ROUND(declared_value * COALESCE(insurance_rate, 0), 2)) VIRTUAL
                    COMMENT 'Virtual generated column: derived cover, never stored',
  gross_weight_kg   DECIMAL(12,3) NOT NULL COMMENT 'Sum of the cargo items, kilograms',
  volume_cbm        DECIMAL(10,3) DEFAULT NULL COMMENT 'Cubic metres',
  notification_flags BIT(8) NOT NULL DEFAULT b'00000001',
  is_hazardous      BOOLEAN NOT NULL DEFAULT FALSE,
  marks_and_numbers TINYTEXT DEFAULT NULL COMMENT 'Shipping marks stencilled on the packages',
  goods_description TEXT NOT NULL COMMENT 'Description of goods as it will print on the bill of lading',
  packing_list      MEDIUMTEXT DEFAULT NULL,
  terms_and_conditions LONGTEXT DEFAULT NULL COMMENT 'Carrier terms in force at the moment of booking',
  booked_at         DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  updated_at        TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  PRIMARY KEY (consignment_id),
  UNIQUE KEY uq_consignment_booking_reference (booking_reference),
  UNIQUE KEY uq_consignment_bl (bill_of_lading_no),
  KEY idx_consignment_shipper_status (shipper_id, status),
  KEY idx_consignment_voyage (vessel_imo, voyage_number),
  KEY idx_consignment_booked_at_desc (booked_at DESC),
  KEY idx_consignment_consignee (consignee_id),
  CONSTRAINT fk_consignment_shipper FOREIGN KEY (shipper_id) REFERENCES customer (customer_id)
    ON DELETE NO ACTION ON UPDATE NO ACTION,
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
-- ISO 6346 equipment types. Small, stable lookup; the referencing column
-- defaults to the dry twenty-foot box, which is what makes ON DELETE SET
-- DEFAULT expressible against it.

CREATE TABLE container_type (
  iso_code        CHAR(4) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'ISO 6346 size-and-type code, e.g. 22G1',
  description     VARCHAR(80) NOT NULL,
  length_ft       TINYINT UNSIGNED NOT NULL,
  height_ft_in    VARCHAR(6) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT '8''6"'
                  COMMENT 'Nominal external height, feet and inches',
  tare_weight_kg  DECIMAL(8,2) NOT NULL,
  max_payload_kg  DECIMAL(8,2) NOT NULL,
  internal_volume_cbm DECIMAL(7,3) NOT NULL,
  is_reefer       BOOLEAN NOT NULL DEFAULT FALSE,
  is_tank         BOOLEAN NOT NULL DEFAULT FALSE,
  PRIMARY KEY (iso_code),
  CONSTRAINT chk_container_type_payload CHECK (max_payload_kg > tare_weight_kg)
) ENGINE=InnoDB
  COMMENT='Tipos de contentor normalizados ISO 6346';

-- -----------------------------------------------------------------------------
-- container
-- -----------------------------------------------------------------------------

CREATE TABLE container (
  container_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  equipment_no    CHAR(11) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                  COMMENT 'ISO 6346 equipment number: four-letter prefix, six digits, one check digit',
  consignment_id  BIGINT UNSIGNED NOT NULL,
  iso_code        CHAR(4) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT '22G1'
                  COMMENT 'Defaults to the standard dry twenty-foot box',
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
    ON DELETE SET DEFAULT ON UPDATE SET DEFAULT,
  CONSTRAINT chk_container_humidity CHECK (humidity_pct IS NULL OR humidity_pct <= 100),
  CONSTRAINT chk_container_cargo_weight CHECK (cargo_weight_kg >= 0)
) ENGINE=InnoDB
  COMMENT='Contentores afectos a cada expedição, com massa bruta verificada (VGM)';

-- -----------------------------------------------------------------------------
-- cargo_item
-- -----------------------------------------------------------------------------
-- Carries the FULLTEXT index and the virtual chargeable-weight column that the
-- rating engine reads.

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
-- SYSTEM VERSIONED. A rate that was in force when a booking was priced must
-- stay retrievable after the rate card is replaced, which is exactly what
-- system versioning is for. Reported by the catalogue with TABLE_TYPE
-- 'SYSTEM VERSIONED' rather than 'BASE TABLE'.
--
-- No foreign key touches this table in either direction: the history rows would
-- have to satisfy the constraint too, and the point of the table is that they
-- outlive the rows they refer to.

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
  WITH SYSTEM VERSIONING
  COMMENT='Tabela de fretes com versionamento de sistema: uma tarifa substituída continua consultável';

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
  KEY idx_charge_invoice (invoice_number),
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
-- Second table, after voyage_leg and port_facility, that points at port.

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
-- Carries the binary surface: the whole BLOB size ladder plus fixed and
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
  signature_thumbnail TINYBLOB DEFAULT NULL COMMENT 'Rendered signature strip, always small',
  page_preview    BLOB DEFAULT NULL COMMENT 'First-page raster preview',
  scanned_original MEDIUMBLOB DEFAULT NULL COMMENT 'Full colour scan of the paper original',
  archive_bundle  LONGBLOB DEFAULT NULL COMMENT 'Signed archival package retained for the statutory period',
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
-- Written by the triggers on consignment. No foreign key: the trail has to
-- outlive the rows it records, including the deleted ones.

CREATE TABLE audit_event (
  audit_event_id  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  entity_name     VARCHAR(64) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  entity_key      VARCHAR(64) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL,
  action          ENUM('INSERT','UPDATE','DELETE') NOT NULL,
  trigger_timing  ENUM('BEFORE','AFTER') NOT NULL,
  changed_column  VARCHAR(64) CHARACTER SET ascii COLLATE ascii_general_ci DEFAULT NULL,
  old_value       VARCHAR(255) DEFAULT NULL,
  new_value       VARCHAR(255) DEFAULT NULL,
  actor           VARCHAR(96) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL DEFAULT 'unknown',
  client_ipv4     INET4 DEFAULT NULL,
  client_ipv6     INET6 DEFAULT NULL,
  correlation_uuid UUID DEFAULT NULL,
  observed_at     DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  detail          TEXT DEFAULT NULL,
  PRIMARY KEY (audit_event_id),
  KEY idx_audit_entity (entity_name, entity_key),
  KEY idx_audit_observed_desc (observed_at DESC)
) ENGINE=InnoDB
  COMMENT='Trilho de auditoria escrito pelos gatilhos; sobrevive à remoção das linhas que regista';

-- -----------------------------------------------------------------------------
-- legacy_edi_field
-- -----------------------------------------------------------------------------
-- IDENTIFIER EDGE CASES. This table exists to exercise identifier quoting and
-- nothing else. Its column names were carried over verbatim from the EDIFACT
-- mapping table of the forwarding system this schema replaced, which allowed a
-- backtick, an embedded space, a reserved word and non-ASCII letters in a field
-- name. Any tool that reads this catalogue must quote and unquote all four
-- correctly; a backtick inside an identifier is written by doubling it.

CREATE TABLE legacy_edi_field (
  mapping_id            SMALLINT UNSIGNED NOT NULL AUTO_INCREMENT,
  segment_tag           CHAR(3) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
                        COMMENT 'EDIFACT segment tag, e.g. BGM, NAD, CNI',
  `back``tick`          VARCHAR(48) DEFAULT NULL
                        COMMENT 'Identifier containing an embedded backtick, doubled when quoted',
  `space in name`       VARCHAR(48) DEFAULT NULL
                        COMMENT 'Identifier containing a space',
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

-- =============================================================================
-- Views
-- =============================================================================

-- Simple projection over a single table. Updatable by construction, with no
-- check option, and declared SQL SECURITY INVOKER so that the two security
-- kinds are both present in the catalogue.
CREATE SQL SECURITY INVOKER VIEW v_port_directory AS
  SELECT
    p.port_id,
    p.un_locode,
    p.name,
    p.country_code,
    p.max_draught_m,
    p.berth_count
  FROM port AS p;

-- Updatable view with a restricting predicate and no check option: a row
-- updated through it may drop out of it, which is precisely the difference
-- from the two below.
CREATE VIEW v_active_vessel AS
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

-- WITH LOCAL CHECK OPTION.
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

-- WITH CASCADED CHECK OPTION, layered over the view above so that both values
-- of the catalogue's check-option attribute are observable.
CREATE VIEW v_reefer_container_deep_frozen AS
  SELECT
    r.container_id,
    r.equipment_no,
    r.consignment_id,
    r.setpoint_celsius
  FROM v_reefer_container AS r
  WHERE r.setpoint_celsius <= -18.0
  WITH CASCADED CHECK OPTION;

-- Multi-table join: not updatable, and the one a template is most likely to
-- render.
CREATE VIEW v_consignment_manifest AS
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
-- Four functions, four distinct return types: DECIMAL, CHAR, BOOLEAN (stored as
-- TINYINT(1)) and VARCHAR. A stored function's parameters are always IN, which
-- is why IN, OUT and INOUT are demonstrated by the procedures below.

DELIMITER $$

CREATE FUNCTION fn_volumetric_weight_kg(p_volume_cbm DECIMAL(10,3))
  RETURNS DECIMAL(12,3)
  DETERMINISTIC
  NO SQL
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
  NO SQL
  COMMENT 'The ISO 3166-1 alpha-2 country prefix of a UN/LOCODE'
BEGIN
  IF p_locode IS NULL OR CHAR_LENGTH(p_locode) < 2 THEN
    RETURN NULL;
  END IF;
  RETURN UPPER(LEFT(p_locode, 2));
END$$

CREATE FUNCTION fn_is_hazardous(p_consignment_id BIGINT UNSIGNED)
  RETURNS BOOLEAN
  READS SQL DATA
  COMMENT 'TRUE when any cargo item on the consignment carries an IMDG class'
BEGIN
  DECLARE v_count INT DEFAULT 0;
  SELECT COUNT(*) INTO v_count
    FROM cargo_item
   WHERE consignment_id = p_consignment_id
     AND imdg_class <> 'Not regulated';
  RETURN v_count > 0;
END$$

CREATE FUNCTION fn_next_booking_reference()
  RETURNS VARCHAR(16)
  NOT DETERMINISTIC
  MODIFIES SQL DATA
  COMMENT 'Draws the next value from booking_reference_seq and formats it'
BEGIN
  DECLARE v_next BIGINT;
  SET v_next = NEXT VALUE FOR booking_reference_seq;
  RETURN CONCAT('BKG-', LPAD(v_next, 9, '0'));
END$$

-- =============================================================================
-- Stored procedures
-- =============================================================================
-- Between them these cover all three parameter modes.

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
  MODIFIES SQL DATA
  COMMENT 'Creates a draft consignment and returns its surrogate key and booking reference'
BEGIN
  SET p_booking_reference = fn_next_booking_reference();
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
  READS SQL DATA
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
  READS SQL DATA
  COMMENT 'Counts a vessel port calls in a rolling window; clamps and returns the window actually used'
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
-- Six triggers on consignment: BEFORE and AFTER for each of INSERT, UPDATE and
-- DELETE.

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

DELIMITER ;

-- =============================================================================
-- Users
-- =============================================================================
-- Two accounts, deliberately unequal.
--
--   root         full privileges; sees every definition.
--   tpl_reader   SELECT and EXECUTE on freight.* and nothing else.
--
-- tpl_reader can list the tables and read their rows, and it can call the
-- routines, so they appear to it in the catalogue. What it cannot do is read a
-- definition: it holds neither SHOW VIEW, so VIEW_DEFINITION comes back empty,
-- nor the privilege that exposes a routine body, so ROUTINE_DEFINITION comes
-- back empty too. That is the point of the account — it makes an incomplete
-- read reproducible without having to break anything.

CREATE USER IF NOT EXISTS 'tpl_reader'@'%' IDENTIFIED BY 'tpl-reader-pw';
GRANT SELECT, EXECUTE ON freight.* TO 'tpl_reader'@'%';
FLUSH PRIVILEGES;
