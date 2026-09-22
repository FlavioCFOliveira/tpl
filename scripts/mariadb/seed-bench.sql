-- =============================================================================
-- tpl benchmark fixture — the reference workloads of NFR-PERF-001
-- =============================================================================
--
-- Two schemas, one per reference workload of
-- `specification/performance-requirements.md`:
--
--   freight_wl001   WL-001, the large workload: 200 tables, 2 400 columns,
--                   600 indexes, 180 foreign keys, 40 generated columns,
--                   25 triggers, 30 views, 40 routines, and a comment on 120
--                   of the 200 tables (60%).
--   freight_wl003   WL-003, the small workload: one table, 12 columns and
--                   3 indexes.
--
-- NFR-PERF-001 requires the number of catalogue statements a full read issues
-- over WL-001 to equal the number it issues over WL-003. The two schemas exist
-- side by side so that the comparison is one run of the same binary against two
-- databases on the same server, with nothing else changed.
--
-- WHY THIS FILE IS SEPARATE FROM seed.sql. BR-PERF-002 keeps the two apart:
-- `setup.sql` and `seed.sql` are exhaustive variety at minimal volume, for
-- correctness, and this file is volume at minimal variety, for measurement.
-- Merging them would hide an N+1 — invisible at 23 objects — or would make the
-- correctness suite pay for 200 tables on every run.
--
-- WHY IT IS NOT IN THE IMAGE. The Dockerfile copies `setup.sql` and `seed.sql`
-- into /docker-entrypoint-initdb.d, so every container carries `freight` from
-- the moment it starts. This file is loaded on demand by `./seed-bench.sh`,
-- for the same reason: a correctness run must not pay for the benchmark
-- workload.
--
-- WHY THERE ARE NO ROWS. Both workloads are defined by catalogue volume, and
-- every measurement stated over them reads INFORMATION_SCHEMA. A row changes no
-- count this file is answerable for, and 200 tables of rows would cost every
-- load without being measured. What the file seeds is the catalogue.
--
-- ACCEPTED BY ALL FOUR SERIES OF FR-SRV-015. Nothing here is version-gated and
-- nothing is conditional, on the same terms as `setup.sql`: every construct is
-- accepted unchanged by 10.11, 11.4, 11.8 and 12.3, so a difference observed
-- between two servers is a difference between the servers.
--
-- THE DOMAIN. A freight-forwarding group's operational system, in thirteen
-- modules: reference data, the physical network, trading parties, the fleet,
-- the commercial pipeline, execution, the warehouse, customs, finance,
-- procurement, people, documents and EDI, and assurance. Every table, column
-- and routine is named for what it holds. The structure is uniform by design —
-- that is what BR-PERF-002 asks of this file — and the names are not.
--
-- HOW TO LOAD IT, AND HOW TO CHECK IT. `./seed-bench.sh` loads this file into
-- each requested server and then counts the nine quantities WL-001 states and
-- the three WL-003 states, per server. `scripts/mariadb/README.md` records the
-- counting rules and the output of a run.
--
-- The file needs a client that honours DELIMITER, because the routines below
-- carry compound bodies. The `mariadb` client does, and `seed-bench.sh` uses it.
-- =============================================================================

SET NAMES utf8mb4;


-- =============================================================================
-- WL-001 — the large workload, in `freight_wl001`
-- =============================================================================

DROP DATABASE IF EXISTS freight_wl001;
CREATE DATABASE freight_wl001
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_520_ci;
USE freight_wl001;

-- The collation is `setup.sql`'s and is chosen for the same reason: it is
-- available on all four series and is the default on none, so a stated schema
-- collation is distinguishable from an inherited one.
--
-- Every table below has the same skeleton — a surrogate primary key, at most
-- one foreign key to its parent, its own columns, and the two audit
-- timestamps — and the nine counts WL-001 fixes are met by that skeleton
-- rather than by a special case anywhere in it:
--
--   200 tables            one CREATE TABLE each
--   2 400 columns         1 primary key + 180 foreign keys + 1 580 own
--                         columns + 40 generated + 400 audit timestamps
--   600 indexes           200 primary + 200 secondary + 20 composite
--                         + 180 foreign-key indexes
--   180 foreign keys      every table but the twenty root lookups has one
--   40 generated columns  every fifth table, alternating STORED and VIRTUAL
--
-- An index on a foreign-key column is declared explicitly, before the
-- constraint that needs it, so that InnoDB adopts it instead of creating one of
-- its own: the 180 foreign keys contribute exactly 180 indexes and not 360.

-- -----------------------------------------------------------------------------
-- Tables
-- -----------------------------------------------------------------------------

CREATE TABLE country (
  country_id                 BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  code                       VARCHAR(16),
  name                       VARCHAR(120),
  short_name                 VARCHAR(48),
  iso_alpha3                 CHAR(3),
  numeric_code               SMALLINT UNSIGNED,
  numeric_code_rounded       DECIMAL(18,2) AS (ROUND(numeric_code, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (country_id),
  UNIQUE KEY ux_country_code (code),
  KEY idx_country_code_created (code, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Reference data shared by every module: country';

CREATE TABLE currency (
  currency_id                BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  iso_alpha3                 CHAR(3),
  numeric_code               SMALLINT UNSIGNED,
  sort_order                 SMALLINT UNSIGNED,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  valid_from                 DATE,
  valid_to                   DATE,
  description                VARCHAR(255),
  notes                      TEXT,
  external_reference         VARCHAR(64),
  revision                   INT UNSIGNED NOT NULL DEFAULT 1,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (currency_id),
  KEY idx_currency_iso_alpha3 (iso_alpha3)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Reference data shared by every module: currency';

CREATE TABLE unit_of_measure (
  unit_of_measure_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  valid_from                 DATE,
  valid_to                   DATE,
  description                VARCHAR(255),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (unit_of_measure_id),
  KEY idx_unit_of_measure_description (description)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Reference data shared by every module: unit of measure';

CREATE TABLE incoterm (
  incoterm_id                BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  description                VARCHAR(255),
  notes                      TEXT,
  external_reference         VARCHAR(64),
  revision                   INT UNSIGNED NOT NULL DEFAULT 1,
  source_system              VARCHAR(32),
  locale_tag                 VARCHAR(16),
  attributes                 JSON,
  review_date                DATE,
  owner_team                 VARCHAR(48),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (incoterm_id),
  KEY idx_incoterm_description (description)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE hs_chapter (
  hs_chapter_id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  revision                   INT UNSIGNED NOT NULL DEFAULT 1,
  source_system              VARCHAR(32),
  locale_tag                 VARCHAR(16),
  attributes                 JSON,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (hs_chapter_id),
  KEY idx_hs_chapter_source_system (source_system)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE hazard_class (
  hazard_class_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  numeric_code               SMALLINT UNSIGNED,
  sort_order                 SMALLINT UNSIGNED,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  valid_from                 DATE,
  valid_to                   DATE,
  description                VARCHAR(255),
  notes                      TEXT,
  external_reference         VARCHAR(64),
  numeric_code_rounded       DECIMAL(18,2) AS (ROUND(numeric_code, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (hazard_class_id),
  KEY idx_hazard_class_description (description)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Reference data shared by every module: hazard class';

CREATE TABLE packaging_type (
  packaging_type_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  iso_alpha3                 CHAR(3),
  numeric_code               SMALLINT UNSIGNED,
  sort_order                 SMALLINT UNSIGNED,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (packaging_type_id),
  KEY idx_packaging_type_iso_alpha3 (iso_alpha3)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Reference data shared by every module: packaging type';

CREATE TABLE container_type (
  container_type_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  description                VARCHAR(255),
  notes                      TEXT,
  external_reference         VARCHAR(64),
  revision                   INT UNSIGNED NOT NULL DEFAULT 1,
  source_system              VARCHAR(32),
  locale_tag                 VARCHAR(16),
  attributes                 JSON,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (container_type_id),
  KEY idx_container_type_description (description)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Reference data shared by every module: container type';

CREATE TABLE vessel_class (
  vessel_class_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  iso_alpha3                 CHAR(3),
  numeric_code               SMALLINT UNSIGNED,
  sort_order                 SMALLINT UNSIGNED,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  valid_from                 DATE,
  valid_to                   DATE,
  description                VARCHAR(255),
  notes                      TEXT,
  external_reference         VARCHAR(64),
  revision                   INT UNSIGNED NOT NULL DEFAULT 1,
  source_system              VARCHAR(32),
  locale_tag                 VARCHAR(16),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (vessel_class_id),
  KEY idx_vessel_class_iso_alpha3 (iso_alpha3)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE trade_lane (
  trade_lane_id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  name                       VARCHAR(120),
  short_name                 VARCHAR(48),
  iso_alpha3                 CHAR(3),
  numeric_code               SMALLINT UNSIGNED,
  sort_order                 SMALLINT UNSIGNED,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (trade_lane_id),
  KEY idx_trade_lane_name (name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE tax_jurisdiction (
  tax_jurisdiction_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  valid_from                 DATE,
  valid_to                   DATE,
  description                VARCHAR(255),
  notes                      TEXT,
  external_reference         VARCHAR(64),
  revision                   INT UNSIGNED NOT NULL DEFAULT 1,
  source_system              VARCHAR(32),
  locale_tag                 VARCHAR(16),
  attributes                 JSON,
  review_date                DATE,
  revision_rounded           DECIMAL(18,2) AS (ROUND(revision, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (tax_jurisdiction_id),
  KEY idx_tax_jurisdiction_description (description),
  KEY idx_tax_jurisdiction_description_created (description, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Reference data shared by every module: tax jurisdiction';

CREATE TABLE document_type (
  document_type_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  sort_order                 SMALLINT UNSIGNED,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  valid_from                 DATE,
  valid_to                   DATE,
  description                VARCHAR(255),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (document_type_id),
  KEY idx_document_type_description (description)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Reference data shared by every module: document type';

CREATE TABLE charge_code (
  charge_code_id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  code                       VARCHAR(16),
  name                       VARCHAR(120),
  short_name                 VARCHAR(48),
  iso_alpha3                 CHAR(3),
  numeric_code               SMALLINT UNSIGNED,
  sort_order                 SMALLINT UNSIGNED,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  valid_from                 DATE,
  valid_to                   DATE,
  description                VARCHAR(255),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (charge_code_id),
  UNIQUE KEY ux_charge_code_code (code)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Reference data shared by every module: charge code';

CREATE TABLE service_level (
  service_level_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  description                VARCHAR(255),
  notes                      TEXT,
  external_reference         VARCHAR(64),
  revision                   INT UNSIGNED NOT NULL DEFAULT 1,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (service_level_id),
  KEY idx_service_level_description (description)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE claim_category (
  claim_category_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  short_name                 VARCHAR(48),
  iso_alpha3                 CHAR(3),
  numeric_code               SMALLINT UNSIGNED,
  sort_order                 SMALLINT UNSIGNED,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  valid_from                 DATE,
  valid_to                   DATE,
  description                VARCHAR(255),
  notes                      TEXT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (claim_category_id),
  KEY idx_claim_category_short_name (short_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE equipment_grade (
  equipment_grade_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  code                       VARCHAR(16),
  name                       VARCHAR(120),
  short_name                 VARCHAR(48),
  iso_alpha3                 CHAR(3),
  code_upper                 VARCHAR(24) AS (UPPER(LEFT(code, 24))) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (equipment_grade_id),
  UNIQUE KEY ux_equipment_grade_code (code)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Reference data shared by every module: equipment grade';

CREATE TABLE spoken_language (
  spoken_language_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  numeric_code               SMALLINT UNSIGNED,
  sort_order                 SMALLINT UNSIGNED,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  valid_from                 DATE,
  valid_to                   DATE,
  description                VARCHAR(255),
  notes                      TEXT,
  external_reference         VARCHAR(64),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (spoken_language_id),
  KEY idx_spoken_language_description (description)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Reference data shared by every module: spoken language';

CREATE TABLE time_zone (
  time_zone_id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  valid_from                 DATE,
  valid_to                   DATE,
  description                VARCHAR(255),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (time_zone_id),
  KEY idx_time_zone_description (description)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Reference data shared by every module: time zone';

CREATE TABLE edi_standard (
  edi_standard_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  valid_from                 DATE,
  valid_to                   DATE,
  description                VARCHAR(255),
  notes                      TEXT,
  external_reference         VARCHAR(64),
  revision                   INT UNSIGNED NOT NULL DEFAULT 1,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (edi_standard_id),
  KEY idx_edi_standard_description (description)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE audit_action (
  audit_action_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  name                       VARCHAR(120),
  short_name                 VARCHAR(48),
  iso_alpha3                 CHAR(3),
  numeric_code               SMALLINT UNSIGNED,
  sort_order                 SMALLINT UNSIGNED,
  is_active                  TINYINT(1) NOT NULL DEFAULT 1,
  valid_from                 DATE,
  valid_to                   DATE,
  description                VARCHAR(255),
  notes                      TEXT,
  external_reference         VARCHAR(64),
  revision                   INT UNSIGNED NOT NULL DEFAULT 1,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (audit_action_id),
  KEY idx_audit_action_name (name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE port (
  port_id                    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  country_id                 BIGINT UNSIGNED NOT NULL,
  un_locode                  CHAR(5),
  name                       VARCHAR(120),
  latitude                   DECIMAL(9,6),
  longitude                  DECIMAL(9,6),
  max_draught_m              DECIMAL(5,2),
  quay_length_m              DECIMAL(7,2),
  latitude_rounded           DECIMAL(18,2) AS (ROUND(latitude, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (port_id),
  UNIQUE KEY ux_port_un_locode (un_locode),
  KEY idx_port_un_locode_created (un_locode, created_at),
  KEY fk_port_country (country_id),
  CONSTRAINT fk_port_country FOREIGN KEY (country_id)
    REFERENCES country (country_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The physical network the group moves cargo through: port';

CREATE TABLE port_terminal (
  port_terminal_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  port_id                    BIGINT UNSIGNED NOT NULL,
  longitude                  DECIMAL(9,6),
  max_draught_m              DECIMAL(5,2),
  quay_length_m              DECIMAL(7,2),
  capacity_teu               INT UNSIGNED,
  opening_time               TIME,
  closing_time               TIME,
  operates_around_clock      TINYINT(1) NOT NULL DEFAULT 0,
  contact_phone              VARCHAR(32),
  approach_notes             TEXT,
  surface_type               VARCHAR(32),
  commissioned_on            DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (port_terminal_id),
  KEY idx_port_terminal_contact_phone (contact_phone),
  KEY fk_port_terminal_port (port_id),
  CONSTRAINT fk_port_terminal_port FOREIGN KEY (port_id)
    REFERENCES port (port_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The physical network the group moves cargo through: port terminal';

CREATE TABLE berth (
  berth_id                   BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  port_terminal_id           BIGINT UNSIGNED NOT NULL,
  capacity_teu               INT UNSIGNED,
  opening_time               TIME,
  closing_time               TIME,
  operates_around_clock      TINYINT(1) NOT NULL DEFAULT 0,
  contact_phone              VARCHAR(32),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (berth_id),
  KEY idx_berth_contact_phone (contact_phone),
  KEY fk_berth_port_terminal (port_terminal_id),
  CONSTRAINT fk_berth_port_terminal FOREIGN KEY (port_terminal_id)
    REFERENCES port_terminal (port_terminal_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The physical network the group moves cargo through: berth';

CREATE TABLE quay_crane (
  quay_crane_id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  port_terminal_id           BIGINT UNSIGNED NOT NULL,
  un_locode                  CHAR(5),
  name                       VARCHAR(120),
  latitude                   DECIMAL(9,6),
  longitude                  DECIMAL(9,6),
  max_draught_m              DECIMAL(5,2),
  quay_length_m              DECIMAL(7,2),
  capacity_teu               INT UNSIGNED,
  opening_time               TIME,
  closing_time               TIME,
  operates_around_clock      TINYINT(1) NOT NULL DEFAULT 0,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (quay_crane_id),
  UNIQUE KEY ux_quay_crane_un_locode (un_locode),
  KEY fk_quay_crane_port_terminal (port_terminal_id),
  CONSTRAINT fk_quay_crane_port_terminal FOREIGN KEY (port_terminal_id)
    REFERENCES port_terminal (port_terminal_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE port_facility (
  port_facility_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  port_id                    BIGINT UNSIGNED NOT NULL,
  surface_type               VARCHAR(32),
  commissioned_on            DATE,
  decommissioned_on          DATE,
  gate_count                 SMALLINT UNSIGNED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (port_facility_id),
  KEY idx_port_facility_surface_type (surface_type),
  KEY fk_port_facility_port (port_id),
  CONSTRAINT fk_port_facility_port FOREIGN KEY (port_id)
    REFERENCES port (port_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE port_restriction (
  port_restriction_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  port_id                    BIGINT UNSIGNED NOT NULL,
  quay_length_m              DECIMAL(7,2),
  capacity_teu               INT UNSIGNED,
  opening_time               TIME,
  closing_time               TIME,
  operates_around_clock      TINYINT(1) NOT NULL DEFAULT 0,
  contact_phone              VARCHAR(32),
  approach_notes             TEXT,
  surface_type               VARCHAR(32),
  commissioned_on            DATE,
  quay_length_m_rounded      DECIMAL(18,2) AS (ROUND(quay_length_m, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (port_restriction_id),
  KEY idx_port_restriction_contact_phone (contact_phone),
  KEY fk_port_restriction_port (port_id),
  CONSTRAINT fk_port_restriction_port FOREIGN KEY (port_id)
    REFERENCES port (port_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The physical network the group moves cargo through: port restriction';

CREATE TABLE locode_alias (
  locode_alias_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  port_id                    BIGINT UNSIGNED NOT NULL,
  longitude                  DECIMAL(9,6),
  max_draught_m              DECIMAL(5,2),
  quay_length_m              DECIMAL(7,2),
  capacity_teu               INT UNSIGNED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (locode_alias_id),
  KEY idx_locode_alias_longitude (longitude),
  KEY fk_locode_alias_port (port_id),
  CONSTRAINT fk_locode_alias_port FOREIGN KEY (port_id)
    REFERENCES port (port_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The physical network the group moves cargo through: locode alias';

CREATE TABLE inland_depot (
  inland_depot_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  country_id                 BIGINT UNSIGNED NOT NULL,
  contact_phone              VARCHAR(32),
  approach_notes             TEXT,
  surface_type               VARCHAR(32),
  commissioned_on            DATE,
  decommissioned_on          DATE,
  gate_count                 SMALLINT UNSIGNED,
  security_level             VARCHAR(24),
  geofence                   JSON,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (inland_depot_id),
  KEY idx_inland_depot_contact_phone (contact_phone),
  KEY fk_inland_depot_country (country_id),
  CONSTRAINT fk_inland_depot_country FOREIGN KEY (country_id)
    REFERENCES country (country_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The physical network the group moves cargo through: inland depot';

CREATE TABLE rail_ramp (
  rail_ramp_id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  inland_depot_id            BIGINT UNSIGNED NOT NULL,
  operates_around_clock      TINYINT(1) NOT NULL DEFAULT 0,
  contact_phone              VARCHAR(32),
  approach_notes             TEXT,
  surface_type               VARCHAR(32),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (rail_ramp_id),
  KEY idx_rail_ramp_contact_phone (contact_phone),
  KEY fk_rail_ramp_inland_depot (inland_depot_id),
  CONSTRAINT fk_rail_ramp_inland_depot FOREIGN KEY (inland_depot_id)
    REFERENCES inland_depot (inland_depot_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE road_corridor (
  road_corridor_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  country_id                 BIGINT UNSIGNED NOT NULL,
  longitude                  DECIMAL(9,6),
  max_draught_m              DECIMAL(5,2),
  quay_length_m              DECIMAL(7,2),
  capacity_teu               INT UNSIGNED,
  opening_time               TIME,
  closing_time               TIME,
  operates_around_clock      TINYINT(1) NOT NULL DEFAULT 0,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (road_corridor_id),
  KEY idx_road_corridor_longitude (longitude),
  KEY fk_road_corridor_country (country_id),
  CONSTRAINT fk_road_corridor_country FOREIGN KEY (country_id)
    REFERENCES country (country_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE customs_office (
  customs_office_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  country_id                 BIGINT UNSIGNED NOT NULL,
  latitude                   DECIMAL(9,6),
  longitude                  DECIMAL(9,6),
  max_draught_m              DECIMAL(5,2),
  quay_length_m              DECIMAL(7,2),
  capacity_teu               INT UNSIGNED,
  opening_time               TIME,
  closing_time               TIME,
  operates_around_clock      TINYINT(1) NOT NULL DEFAULT 0,
  contact_phone              VARCHAR(32),
  approach_notes             TEXT,
  surface_type               VARCHAR(32),
  commissioned_on            DATE,
  latitude_rounded           DECIMAL(18,2) AS (ROUND(latitude, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (customs_office_id),
  KEY idx_customs_office_contact_phone (contact_phone),
  KEY idx_customs_office_contact_phone_created (contact_phone, created_at),
  KEY fk_customs_office_country (country_id),
  CONSTRAINT fk_customs_office_country FOREIGN KEY (country_id)
    REFERENCES country (country_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The physical network the group moves cargo through: customs office';

CREATE TABLE free_zone (
  free_zone_id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  country_id                 BIGINT UNSIGNED NOT NULL,
  opening_time               TIME,
  closing_time               TIME,
  operates_around_clock      TINYINT(1) NOT NULL DEFAULT 0,
  contact_phone              VARCHAR(32),
  approach_notes             TEXT,
  surface_type               VARCHAR(32),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (free_zone_id),
  KEY idx_free_zone_contact_phone (contact_phone),
  KEY fk_free_zone_country (country_id),
  CONSTRAINT fk_free_zone_country FOREIGN KEY (country_id)
    REFERENCES country (country_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The physical network the group moves cargo through: free zone';

CREATE TABLE warehouse_site (
  warehouse_site_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  country_id                 BIGINT UNSIGNED NOT NULL,
  max_draught_m              DECIMAL(5,2),
  quay_length_m              DECIMAL(7,2),
  capacity_teu               INT UNSIGNED,
  opening_time               TIME,
  closing_time               TIME,
  operates_around_clock      TINYINT(1) NOT NULL DEFAULT 0,
  contact_phone              VARCHAR(32),
  approach_notes             TEXT,
  surface_type               VARCHAR(32),
  commissioned_on            DATE,
  decommissioned_on          DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (warehouse_site_id),
  KEY idx_warehouse_site_contact_phone (contact_phone),
  KEY fk_warehouse_site_country (country_id),
  CONSTRAINT fk_warehouse_site_country FOREIGN KEY (country_id)
    REFERENCES country (country_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The physical network the group moves cargo through: warehouse site';

CREATE TABLE warehouse_zone (
  warehouse_zone_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  warehouse_site_id          BIGINT UNSIGNED NOT NULL,
  approach_notes             TEXT,
  surface_type               VARCHAR(32),
  commissioned_on            DATE,
  decommissioned_on          DATE,
  gate_count                 SMALLINT UNSIGNED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (warehouse_zone_id),
  KEY idx_warehouse_zone_surface_type (surface_type),
  KEY fk_warehouse_zone_warehouse_site (warehouse_site_id),
  CONSTRAINT fk_warehouse_zone_warehouse_site FOREIGN KEY (warehouse_site_id)
    REFERENCES warehouse_site (warehouse_site_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE organisation (
  organisation_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  country_id                 BIGINT UNSIGNED NOT NULL,
  legal_name                 VARCHAR(160),
  trading_name               VARCHAR(160),
  registration_no            VARCHAR(48),
  vat_number                 VARCHAR(32),
  street_line1               VARCHAR(120),
  street_line2               VARCHAR(120),
  postal_code                VARCHAR(16),
  city                       VARCHAR(80),
  email                      VARCHAR(160),
  phone                      VARCHAR(32),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (organisation_id),
  KEY idx_organisation_legal_name (legal_name),
  KEY fk_organisation_country (country_id),
  CONSTRAINT fk_organisation_country FOREIGN KEY (country_id)
    REFERENCES country (country_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE organisation_address (
  organisation_address_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  vat_number                 VARCHAR(32),
  street_line1               VARCHAR(120),
  street_line2               VARCHAR(120),
  postal_code                VARCHAR(16),
  vat_number_upper           VARCHAR(24) AS (UPPER(LEFT(vat_number, 24))) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (organisation_address_id),
  KEY idx_organisation_address_vat_number (vat_number),
  KEY fk_organisation_address_organisation (organisation_id),
  CONSTRAINT fk_organisation_address_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The commercial parties the group trades with: organisation address';

CREATE TABLE organisation_contact (
  organisation_contact_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  postal_code                VARCHAR(16),
  city                       VARCHAR(80),
  email                      VARCHAR(160),
  phone                      VARCHAR(32),
  credit_rating              VARCHAR(8),
  credit_limit               DECIMAL(14,2),
  payment_terms_days         SMALLINT UNSIGNED,
  onboarded_on               DATE,
  terminated_on              DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (organisation_contact_id),
  KEY idx_organisation_contact_postal_code (postal_code),
  KEY fk_organisation_contact_organisation (organisation_id),
  CONSTRAINT fk_organisation_contact_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The commercial parties the group trades with: organisation contact';

CREATE TABLE customer (
  customer_id                BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  phone                      VARCHAR(32),
  credit_rating              VARCHAR(8),
  credit_limit               DECIMAL(14,2),
  payment_terms_days         SMALLINT UNSIGNED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (customer_id),
  KEY idx_customer_phone (phone),
  KEY fk_customer_organisation (organisation_id),
  CONSTRAINT fk_customer_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The commercial parties the group trades with: customer';

CREATE TABLE customer_credit_limit (
  customer_credit_limit_id   BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  customer_id                BIGINT UNSIGNED NOT NULL,
  trading_name               VARCHAR(160),
  registration_no            VARCHAR(48),
  vat_number                 VARCHAR(32),
  street_line1               VARCHAR(120),
  street_line2               VARCHAR(120),
  postal_code                VARCHAR(16),
  city                       VARCHAR(80),
  email                      VARCHAR(160),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (customer_credit_limit_id),
  KEY idx_customer_credit_limit_trading_name (trading_name),
  KEY fk_customer_credit_limit_customer (customer_id),
  CONSTRAINT fk_customer_credit_limit_customer FOREIGN KEY (customer_id)
    REFERENCES customer (customer_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE customer_tariff_link (
  customer_tariff_link_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  customer_id                BIGINT UNSIGNED NOT NULL,
  legal_name                 VARCHAR(160),
  trading_name               VARCHAR(160),
  registration_no            VARCHAR(48),
  vat_number                 VARCHAR(32),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (customer_tariff_link_id),
  KEY idx_customer_tariff_link_legal_name (legal_name),
  KEY fk_customer_tariff_link_customer (customer_id),
  CONSTRAINT fk_customer_tariff_link_customer FOREIGN KEY (customer_id)
    REFERENCES customer (customer_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE vendor (
  vendor_id                  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  postal_code                VARCHAR(16),
  city                       VARCHAR(80),
  email                      VARCHAR(160),
  phone                      VARCHAR(32),
  credit_rating              VARCHAR(8),
  credit_limit               DECIMAL(14,2),
  payment_terms_days         SMALLINT UNSIGNED,
  credit_limit_rounded       DECIMAL(18,2) AS (ROUND(credit_limit, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (vendor_id),
  KEY idx_vendor_postal_code (postal_code),
  KEY idx_vendor_postal_code_created (postal_code, created_at),
  KEY fk_vendor_organisation (organisation_id),
  CONSTRAINT fk_vendor_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The commercial parties the group trades with: vendor';

CREATE TABLE vendor_rating (
  vendor_rating_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vendor_id                  BIGINT UNSIGNED NOT NULL,
  legal_name                 VARCHAR(160),
  trading_name               VARCHAR(160),
  registration_no            VARCHAR(48),
  vat_number                 VARCHAR(32),
  street_line1               VARCHAR(120),
  street_line2               VARCHAR(120),
  postal_code                VARCHAR(16),
  city                       VARCHAR(80),
  email                      VARCHAR(160),
  phone                      VARCHAR(32),
  credit_rating              VARCHAR(8),
  credit_limit               DECIMAL(14,2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (vendor_rating_id),
  KEY idx_vendor_rating_legal_name (legal_name),
  KEY fk_vendor_rating_vendor (vendor_id),
  CONSTRAINT fk_vendor_rating_vendor FOREIGN KEY (vendor_id)
    REFERENCES vendor (vendor_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The commercial parties the group trades with: vendor rating';

CREATE TABLE carrier_profile (
  carrier_profile_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  credit_limit               DECIMAL(14,2),
  payment_terms_days         SMALLINT UNSIGNED,
  onboarded_on               DATE,
  terminated_on              DATE,
  preferred_language         VARCHAR(16),
  remarks                    TEXT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (carrier_profile_id),
  KEY idx_carrier_profile_preferred_language (preferred_language),
  KEY fk_carrier_profile_organisation (organisation_id),
  CONSTRAINT fk_carrier_profile_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The commercial parties the group trades with: carrier profile';

CREATE TABLE agent_network (
  agent_network_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  vat_number                 VARCHAR(32),
  street_line1               VARCHAR(120),
  street_line2               VARCHAR(120),
  postal_code                VARCHAR(16),
  city                       VARCHAR(80),
  email                      VARCHAR(160),
  phone                      VARCHAR(32),
  credit_rating              VARCHAR(8),
  credit_limit               DECIMAL(14,2),
  payment_terms_days         SMALLINT UNSIGNED,
  onboarded_on               DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (agent_network_id),
  KEY idx_agent_network_vat_number (vat_number),
  KEY fk_agent_network_organisation (organisation_id),
  CONSTRAINT fk_agent_network_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE notify_party (
  notify_party_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  registration_no            VARCHAR(48),
  vat_number                 VARCHAR(32),
  street_line1               VARCHAR(120),
  street_line2               VARCHAR(120),
  postal_code                VARCHAR(16),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (notify_party_id),
  UNIQUE KEY ux_notify_party_registration_no (registration_no),
  KEY fk_notify_party_organisation (organisation_id),
  CONSTRAINT fk_notify_party_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE consignee (
  consignee_id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  postal_code                VARCHAR(16),
  city                       VARCHAR(80),
  email                      VARCHAR(160),
  phone                      VARCHAR(32),
  credit_rating              VARCHAR(8),
  credit_limit               DECIMAL(14,2),
  payment_terms_days         SMALLINT UNSIGNED,
  onboarded_on               DATE,
  terminated_on              DATE,
  preferred_language         VARCHAR(16),
  credit_limit_rounded       DECIMAL(18,2) AS (ROUND(credit_limit, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (consignee_id),
  KEY idx_consignee_postal_code (postal_code),
  KEY fk_consignee_organisation (organisation_id),
  CONSTRAINT fk_consignee_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The commercial parties the group trades with: consignee';

CREATE TABLE shipper (
  shipper_id                 BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  postal_code                VARCHAR(16),
  city                       VARCHAR(80),
  email                      VARCHAR(160),
  phone                      VARCHAR(32),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (shipper_id),
  KEY idx_shipper_postal_code (postal_code),
  KEY fk_shipper_organisation (organisation_id),
  CONSTRAINT fk_shipper_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The commercial parties the group trades with: shipper';

CREATE TABLE broker_licence (
  broker_licence_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  vat_number                 VARCHAR(32),
  street_line1               VARCHAR(120),
  street_line2               VARCHAR(120),
  postal_code                VARCHAR(16),
  city                       VARCHAR(80),
  email                      VARCHAR(160),
  phone                      VARCHAR(32),
  credit_rating              VARCHAR(8),
  credit_limit               DECIMAL(14,2),
  payment_terms_days         SMALLINT UNSIGNED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (broker_licence_id),
  KEY idx_broker_licence_vat_number (vat_number),
  KEY fk_broker_licence_organisation (organisation_id),
  CONSTRAINT fk_broker_licence_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='The commercial parties the group trades with: broker licence';

CREATE TABLE bank_account (
  bank_account_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  payment_terms_days         SMALLINT UNSIGNED,
  onboarded_on               DATE,
  terminated_on              DATE,
  preferred_language         VARCHAR(16),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (bank_account_id),
  KEY idx_bank_account_preferred_language (preferred_language),
  KEY fk_bank_account_organisation (organisation_id),
  CONSTRAINT fk_bank_account_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE party_document (
  party_document_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  street_line2               VARCHAR(120),
  postal_code                VARCHAR(16),
  city                       VARCHAR(80),
  email                      VARCHAR(160),
  phone                      VARCHAR(32),
  credit_rating              VARCHAR(8),
  credit_limit               DECIMAL(14,2),
  payment_terms_days         SMALLINT UNSIGNED,
  onboarded_on               DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (party_document_id),
  KEY idx_party_document_street_line2 (street_line2),
  KEY fk_party_document_organisation (organisation_id),
  CONSTRAINT fk_party_document_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE vessel (
  vessel_id                  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vessel_class_id            BIGINT UNSIGNED NOT NULL,
  imo_number                 CHAR(7),
  call_sign                  VARCHAR(16),
  name                       VARCHAR(120),
  gross_tonnage              INT UNSIGNED,
  gross_tonnage_rounded      DECIMAL(18,2) AS (ROUND(gross_tonnage, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (vessel_id),
  UNIQUE KEY ux_vessel_imo_number (imo_number),
  KEY idx_vessel_imo_number_created (imo_number, created_at),
  KEY fk_vessel_vessel_class (vessel_class_id),
  CONSTRAINT fk_vessel_vessel_class FOREIGN KEY (vessel_class_id)
    REFERENCES vessel_class (vessel_class_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Vessels, containers and the equipment fleet: vessel';

CREATE TABLE vessel_certificate (
  vessel_certificate_id      BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vessel_id                  BIGINT UNSIGNED NOT NULL,
  gross_tonnage              INT UNSIGNED,
  deadweight_tonnes          INT UNSIGNED,
  length_overall_m           DECIMAL(7,2),
  beam_m                     DECIMAL(6,2),
  service_speed_kn           DECIMAL(4,1),
  built_year                 SMALLINT UNSIGNED,
  flag_state                 CHAR(2),
  classification_society     VARCHAR(64),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (vessel_certificate_id),
  KEY idx_vessel_certificate_flag_state (flag_state),
  KEY fk_vessel_certificate_vessel (vessel_id),
  CONSTRAINT fk_vessel_certificate_vessel FOREIGN KEY (vessel_id)
    REFERENCES vessel (vessel_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Vessels, containers and the equipment fleet: vessel certificate';

CREATE TABLE vessel_particulars (
  vessel_particulars_id      BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vessel_id                  BIGINT UNSIGNED NOT NULL,
  imo_number                 CHAR(7),
  call_sign                  VARCHAR(16),
  name                       VARCHAR(120),
  gross_tonnage              INT UNSIGNED,
  deadweight_tonnes          INT UNSIGNED,
  length_overall_m           DECIMAL(7,2),
  beam_m                     DECIMAL(6,2),
  service_speed_kn           DECIMAL(4,1),
  built_year                 SMALLINT UNSIGNED,
  flag_state                 CHAR(2),
  classification_society     VARCHAR(64),
  next_survey_on             DATE,
  last_drydock_on            DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (vessel_particulars_id),
  UNIQUE KEY ux_vessel_particulars_imo_number (imo_number),
  KEY fk_vessel_particulars_vessel (vessel_id),
  CONSTRAINT fk_vessel_particulars_vessel FOREIGN KEY (vessel_id)
    REFERENCES vessel (vessel_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Vessels, containers and the equipment fleet: vessel particulars';

CREATE TABLE vessel_bunker_tank (
  vessel_bunker_tank_id      BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vessel_id                  BIGINT UNSIGNED NOT NULL,
  flag_state                 CHAR(2),
  classification_society     VARCHAR(64),
  next_survey_on             DATE,
  last_drydock_on            DATE,
  fuel_capacity_t            DECIMAL(9,2),
  reefer_plug_count          SMALLINT UNSIGNED,
  operational_status         VARCHAR(24),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (vessel_bunker_tank_id),
  KEY idx_vessel_bunker_tank_flag_state (flag_state),
  KEY fk_vessel_bunker_tank_vessel (vessel_id),
  CONSTRAINT fk_vessel_bunker_tank_vessel FOREIGN KEY (vessel_id)
    REFERENCES vessel (vessel_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE voyage (
  voyage_id                  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vessel_id                  BIGINT UNSIGNED NOT NULL,
  length_overall_m           DECIMAL(7,2),
  beam_m                     DECIMAL(6,2),
  service_speed_kn           DECIMAL(4,1),
  built_year                 SMALLINT UNSIGNED,
  flag_state                 CHAR(2),
  classification_society     VARCHAR(64),
  next_survey_on             DATE,
  last_drydock_on            DATE,
  fuel_capacity_t            DECIMAL(9,2),
  reefer_plug_count          SMALLINT UNSIGNED,
  operational_status         VARCHAR(24),
  technical_notes            TEXT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (voyage_id),
  KEY idx_voyage_flag_state (flag_state),
  KEY fk_voyage_vessel (vessel_id),
  CONSTRAINT fk_voyage_vessel FOREIGN KEY (vessel_id)
    REFERENCES vessel (vessel_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE voyage_leg (
  voyage_leg_id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  voyage_id                  BIGINT UNSIGNED NOT NULL,
  name                       VARCHAR(120),
  gross_tonnage              INT UNSIGNED,
  deadweight_tonnes          INT UNSIGNED,
  length_overall_m           DECIMAL(7,2),
  beam_m                     DECIMAL(6,2),
  service_speed_kn           DECIMAL(4,1),
  gross_tonnage_rounded      DECIMAL(18,2) AS (ROUND(gross_tonnage, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (voyage_leg_id),
  KEY idx_voyage_leg_name (name),
  KEY fk_voyage_leg_voyage (voyage_id),
  CONSTRAINT fk_voyage_leg_voyage FOREIGN KEY (voyage_id)
    REFERENCES voyage (voyage_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Vessels, containers and the equipment fleet: voyage leg';

CREATE TABLE port_call (
  port_call_id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  voyage_leg_id              BIGINT UNSIGNED NOT NULL,
  name                       VARCHAR(120),
  gross_tonnage              INT UNSIGNED,
  deadweight_tonnes          INT UNSIGNED,
  length_overall_m           DECIMAL(7,2),
  beam_m                     DECIMAL(6,2),
  service_speed_kn           DECIMAL(4,1),
  built_year                 SMALLINT UNSIGNED,
  flag_state                 CHAR(2),
  classification_society     VARCHAR(64),
  next_survey_on             DATE,
  last_drydock_on            DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (port_call_id),
  KEY idx_port_call_name (name),
  KEY fk_port_call_voyage_leg (voyage_leg_id),
  CONSTRAINT fk_port_call_voyage_leg FOREIGN KEY (voyage_leg_id)
    REFERENCES voyage_leg (voyage_leg_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Vessels, containers and the equipment fleet: port call';

CREATE TABLE bunker_stem (
  bunker_stem_id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  voyage_id                  BIGINT UNSIGNED NOT NULL,
  service_speed_kn           DECIMAL(4,1),
  built_year                 SMALLINT UNSIGNED,
  flag_state                 CHAR(2),
  classification_society     VARCHAR(64),
  next_survey_on             DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (bunker_stem_id),
  KEY idx_bunker_stem_flag_state (flag_state),
  KEY fk_bunker_stem_voyage (voyage_id),
  CONSTRAINT fk_bunker_stem_voyage FOREIGN KEY (voyage_id)
    REFERENCES voyage (voyage_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Vessels, containers and the equipment fleet: bunker stem';

CREATE TABLE container_unit (
  container_unit_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  container_type_id          BIGINT UNSIGNED NOT NULL,
  beam_m                     DECIMAL(6,2),
  service_speed_kn           DECIMAL(4,1),
  built_year                 SMALLINT UNSIGNED,
  flag_state                 CHAR(2),
  classification_society     VARCHAR(64),
  next_survey_on             DATE,
  last_drydock_on            DATE,
  fuel_capacity_t            DECIMAL(9,2),
  reefer_plug_count          SMALLINT UNSIGNED,
  operational_status         VARCHAR(24),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (container_unit_id),
  KEY idx_container_unit_flag_state (flag_state),
  KEY fk_container_unit_container_type (container_type_id),
  CONSTRAINT fk_container_unit_container_type FOREIGN KEY (container_type_id)
    REFERENCES container_type (container_type_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE container_inspection (
  container_inspection_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  container_unit_id          BIGINT UNSIGNED NOT NULL,
  last_drydock_on            DATE,
  fuel_capacity_t            DECIMAL(9,2),
  reefer_plug_count          SMALLINT UNSIGNED,
  operational_status         VARCHAR(24),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (container_inspection_id),
  KEY idx_container_inspection_operational_status (operational_status),
  KEY fk_container_inspection_container_unit (container_unit_id),
  CONSTRAINT fk_container_inspection_container_unit FOREIGN KEY (container_unit_id)
    REFERENCES container_unit (container_unit_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE container_repair (
  container_repair_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  container_unit_id          BIGINT UNSIGNED NOT NULL,
  imo_number                 CHAR(7),
  call_sign                  VARCHAR(16),
  name                       VARCHAR(120),
  gross_tonnage              INT UNSIGNED,
  deadweight_tonnes          INT UNSIGNED,
  length_overall_m           DECIMAL(7,2),
  beam_m                     DECIMAL(6,2),
  service_speed_kn           DECIMAL(4,1),
  built_year                 SMALLINT UNSIGNED,
  gross_tonnage_rounded      DECIMAL(18,2) AS (ROUND(gross_tonnage, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (container_repair_id),
  UNIQUE KEY ux_container_repair_imo_number (imo_number),
  KEY idx_container_repair_imo_number_created (imo_number, created_at),
  KEY fk_container_repair_container_unit (container_unit_id),
  CONSTRAINT fk_container_repair_container_unit FOREIGN KEY (container_unit_id)
    REFERENCES container_unit (container_unit_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Vessels, containers and the equipment fleet: container repair';

CREATE TABLE container_lease (
  container_lease_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  container_unit_id          BIGINT UNSIGNED NOT NULL,
  gross_tonnage              INT UNSIGNED,
  deadweight_tonnes          INT UNSIGNED,
  length_overall_m           DECIMAL(7,2),
  beam_m                     DECIMAL(6,2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (container_lease_id),
  KEY idx_container_lease_gross_tonnage (gross_tonnage),
  KEY fk_container_lease_container_unit (container_unit_id),
  CONSTRAINT fk_container_lease_container_unit FOREIGN KEY (container_unit_id)
    REFERENCES container_unit (container_unit_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Vessels, containers and the equipment fleet: container lease';

CREATE TABLE chassis (
  chassis_id                 BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  equipment_grade_id         BIGINT UNSIGNED NOT NULL,
  gross_tonnage              INT UNSIGNED,
  deadweight_tonnes          INT UNSIGNED,
  length_overall_m           DECIMAL(7,2),
  beam_m                     DECIMAL(6,2),
  service_speed_kn           DECIMAL(4,1),
  built_year                 SMALLINT UNSIGNED,
  flag_state                 CHAR(2),
  classification_society     VARCHAR(64),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (chassis_id),
  KEY idx_chassis_flag_state (flag_state),
  KEY fk_chassis_equipment_grade (equipment_grade_id),
  CONSTRAINT fk_chassis_equipment_grade FOREIGN KEY (equipment_grade_id)
    REFERENCES equipment_grade (equipment_grade_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Vessels, containers and the equipment fleet: chassis';

CREATE TABLE genset (
  genset_id                  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  equipment_grade_id         BIGINT UNSIGNED NOT NULL,
  gross_tonnage              INT UNSIGNED,
  deadweight_tonnes          INT UNSIGNED,
  length_overall_m           DECIMAL(7,2),
  beam_m                     DECIMAL(6,2),
  service_speed_kn           DECIMAL(4,1),
  built_year                 SMALLINT UNSIGNED,
  flag_state                 CHAR(2),
  classification_society     VARCHAR(64),
  next_survey_on             DATE,
  last_drydock_on            DATE,
  fuel_capacity_t            DECIMAL(9,2),
  reefer_plug_count          SMALLINT UNSIGNED,
  operational_status         VARCHAR(24),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (genset_id),
  KEY idx_genset_flag_state (flag_state),
  KEY fk_genset_equipment_grade (equipment_grade_id),
  CONSTRAINT fk_genset_equipment_grade FOREIGN KEY (equipment_grade_id)
    REFERENCES equipment_grade (equipment_grade_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE reefer_setpoint (
  reefer_setpoint_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  container_unit_id          BIGINT UNSIGNED NOT NULL,
  beam_m                     DECIMAL(6,2),
  service_speed_kn           DECIMAL(4,1),
  built_year                 SMALLINT UNSIGNED,
  flag_state                 CHAR(2),
  classification_society     VARCHAR(64),
  next_survey_on             DATE,
  last_drydock_on            DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (reefer_setpoint_id),
  KEY idx_reefer_setpoint_flag_state (flag_state),
  KEY fk_reefer_setpoint_container_unit (container_unit_id),
  CONSTRAINT fk_reefer_setpoint_container_unit FOREIGN KEY (container_unit_id)
    REFERENCES container_unit (container_unit_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE equipment_movement (
  equipment_movement_id      BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  container_unit_id          BIGINT UNSIGNED NOT NULL,
  gross_tonnage              INT UNSIGNED,
  deadweight_tonnes          INT UNSIGNED,
  length_overall_m           DECIMAL(7,2),
  beam_m                     DECIMAL(6,2),
  service_speed_kn           DECIMAL(4,1),
  built_year                 SMALLINT UNSIGNED,
  flag_state                 CHAR(2),
  classification_society     VARCHAR(64),
  next_survey_on             DATE,
  last_drydock_on            DATE,
  fuel_capacity_t            DECIMAL(9,2),
  reefer_plug_count          SMALLINT UNSIGNED,
  gross_tonnage_rounded      DECIMAL(18,2) AS (ROUND(gross_tonnage, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (equipment_movement_id),
  KEY idx_equipment_movement_flag_state (flag_state),
  KEY fk_equipment_movement_container_unit (container_unit_id),
  CONSTRAINT fk_equipment_movement_container_unit FOREIGN KEY (container_unit_id)
    REFERENCES container_unit (container_unit_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Vessels, containers and the equipment fleet: equipment movement';

CREATE TABLE quotation (
  quotation_id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  customer_id                BIGINT UNSIGNED NOT NULL,
  reference                  VARCHAR(32),
  status                     VARCHAR(24),
  issued_on                  DATE,
  expires_on                 DATE,
  currency_code              CHAR(3),
  net_amount                 DECIMAL(14,2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (quotation_id),
  UNIQUE KEY ux_quotation_reference (reference),
  KEY fk_quotation_customer (customer_id),
  CONSTRAINT fk_quotation_customer FOREIGN KEY (customer_id)
    REFERENCES customer (customer_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Quotation, contract and booking of cargo: quotation';

CREATE TABLE quotation_line (
  quotation_line_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  quotation_id               BIGINT UNSIGNED NOT NULL,
  expires_on                 DATE,
  currency_code              CHAR(3),
  net_amount                 DECIMAL(14,2),
  discount_pct               DECIMAL(5,2),
  gross_weight_kg            DECIMAL(12,3),
  volume_cbm                 DECIMAL(10,3),
  piece_count                INT UNSIGNED,
  incoterm_code              CHAR(3),
  origin_locode              CHAR(5),
  destination_locode         CHAR(5),
  requested_pickup_on        DATETIME,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (quotation_line_id),
  KEY idx_quotation_line_currency_code (currency_code),
  KEY fk_quotation_line_quotation (quotation_id),
  CONSTRAINT fk_quotation_line_quotation FOREIGN KEY (quotation_id)
    REFERENCES quotation (quotation_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Quotation, contract and booking of cargo: quotation line';

CREATE TABLE rate_card (
  rate_card_id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  trade_lane_id              BIGINT UNSIGNED NOT NULL,
  discount_pct               DECIMAL(5,2),
  gross_weight_kg            DECIMAL(12,3),
  volume_cbm                 DECIMAL(10,3),
  piece_count                INT UNSIGNED,
  incoterm_code              CHAR(3),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (rate_card_id),
  KEY idx_rate_card_incoterm_code (incoterm_code),
  KEY fk_rate_card_trade_lane (trade_lane_id),
  CONSTRAINT fk_rate_card_trade_lane FOREIGN KEY (trade_lane_id)
    REFERENCES trade_lane (trade_lane_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE rate_card_line (
  rate_card_line_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  rate_card_id               BIGINT UNSIGNED NOT NULL,
  reference                  VARCHAR(32),
  status                     VARCHAR(24),
  issued_on                  DATE,
  expires_on                 DATE,
  currency_code              CHAR(3),
  net_amount                 DECIMAL(14,2),
  discount_pct               DECIMAL(5,2),
  gross_weight_kg            DECIMAL(12,3),
  volume_cbm                 DECIMAL(10,3),
  piece_count                INT UNSIGNED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (rate_card_line_id),
  UNIQUE KEY ux_rate_card_line_reference (reference),
  KEY fk_rate_card_line_rate_card (rate_card_id),
  CONSTRAINT fk_rate_card_line_rate_card FOREIGN KEY (rate_card_id)
    REFERENCES rate_card (rate_card_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE surcharge (
  surcharge_id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  charge_code_id             BIGINT UNSIGNED NOT NULL,
  destination_locode         CHAR(5),
  requested_pickup_on        DATETIME,
  promised_delivery_on       DATETIME,
  special_instructions       TEXT,
  requested_pickup_on_year   SMALLINT UNSIGNED AS (YEAR(requested_pickup_on)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (surcharge_id),
  KEY idx_surcharge_destination_locode (destination_locode),
  KEY idx_surcharge_destination_locode_created (destination_locode, created_at),
  KEY fk_surcharge_charge_code (charge_code_id),
  CONSTRAINT fk_surcharge_charge_code FOREIGN KEY (charge_code_id)
    REFERENCES charge_code (charge_code_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Quotation, contract and booking of cargo: surcharge';

CREATE TABLE contract (
  contract_id                BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  customer_id                BIGINT UNSIGNED NOT NULL,
  net_amount                 DECIMAL(14,2),
  discount_pct               DECIMAL(5,2),
  gross_weight_kg            DECIMAL(12,3),
  volume_cbm                 DECIMAL(10,3),
  piece_count                INT UNSIGNED,
  incoterm_code              CHAR(3),
  origin_locode              CHAR(5),
  destination_locode         CHAR(5),
  requested_pickup_on        DATETIME,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (contract_id),
  KEY idx_contract_incoterm_code (incoterm_code),
  KEY fk_contract_customer (customer_id),
  CONSTRAINT fk_contract_customer FOREIGN KEY (customer_id)
    REFERENCES customer (customer_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Quotation, contract and booking of cargo: contract';

CREATE TABLE contract_line (
  contract_line_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  contract_id                BIGINT UNSIGNED NOT NULL,
  expires_on                 DATE,
  currency_code              CHAR(3),
  net_amount                 DECIMAL(14,2),
  discount_pct               DECIMAL(5,2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (contract_line_id),
  KEY idx_contract_line_currency_code (currency_code),
  KEY fk_contract_line_contract (contract_id),
  CONSTRAINT fk_contract_line_contract FOREIGN KEY (contract_id)
    REFERENCES contract (contract_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Quotation, contract and booking of cargo: contract line';

CREATE TABLE booking (
  booking_id                 BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  customer_id                BIGINT UNSIGNED NOT NULL,
  incoterm_code              CHAR(3),
  origin_locode              CHAR(5),
  destination_locode         CHAR(5),
  requested_pickup_on        DATETIME,
  promised_delivery_on       DATETIME,
  special_instructions       TEXT,
  sales_owner                VARCHAR(64),
  line_metadata              JSON,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (booking_id),
  KEY idx_booking_incoterm_code (incoterm_code),
  KEY fk_booking_customer (customer_id),
  CONSTRAINT fk_booking_customer FOREIGN KEY (customer_id)
    REFERENCES customer (customer_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE booking_line (
  booking_line_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  booking_id                 BIGINT UNSIGNED NOT NULL,
  reference                  VARCHAR(32),
  status                     VARCHAR(24),
  issued_on                  DATE,
  expires_on                 DATE,
  currency_code              CHAR(3),
  net_amount                 DECIMAL(14,2),
  discount_pct               DECIMAL(5,2),
  gross_weight_kg            DECIMAL(12,3),
  volume_cbm                 DECIMAL(10,3),
  piece_count                INT UNSIGNED,
  incoterm_code              CHAR(3),
  origin_locode              CHAR(5),
  destination_locode         CHAR(5),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (booking_line_id),
  UNIQUE KEY ux_booking_line_reference (reference),
  KEY fk_booking_line_booking (booking_id),
  CONSTRAINT fk_booking_line_booking FOREIGN KEY (booking_id)
    REFERENCES booking (booking_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE booking_amendment (
  booking_amendment_id       BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  booking_id                 BIGINT UNSIGNED NOT NULL,
  expires_on                 DATE,
  currency_code              CHAR(3),
  net_amount                 DECIMAL(14,2),
  discount_pct               DECIMAL(5,2),
  gross_weight_kg            DECIMAL(12,3),
  volume_cbm                 DECIMAL(10,3),
  piece_count                INT UNSIGNED,
  net_amount_rounded         DECIMAL(18,2) AS (ROUND(net_amount, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (booking_amendment_id),
  KEY idx_booking_amendment_currency_code (currency_code),
  KEY fk_booking_amendment_booking (booking_id),
  CONSTRAINT fk_booking_amendment_booking FOREIGN KEY (booking_id)
    REFERENCES booking (booking_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Quotation, contract and booking of cargo: booking amendment';

CREATE TABLE allocation (
  allocation_id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  booking_id                 BIGINT UNSIGNED NOT NULL,
  issued_on                  DATE,
  expires_on                 DATE,
  currency_code              CHAR(3),
  net_amount                 DECIMAL(14,2),
  discount_pct               DECIMAL(5,2),
  gross_weight_kg            DECIMAL(12,3),
  volume_cbm                 DECIMAL(10,3),
  piece_count                INT UNSIGNED,
  incoterm_code              CHAR(3),
  origin_locode              CHAR(5),
  destination_locode         CHAR(5),
  requested_pickup_on        DATETIME,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (allocation_id),
  KEY idx_allocation_currency_code (currency_code),
  KEY fk_allocation_booking (booking_id),
  CONSTRAINT fk_allocation_booking FOREIGN KEY (booking_id)
    REFERENCES booking (booking_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Quotation, contract and booking of cargo: allocation';

CREATE TABLE shipping_instruction (
  shipping_instruction_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  booking_id                 BIGINT UNSIGNED NOT NULL,
  gross_weight_kg            DECIMAL(12,3),
  volume_cbm                 DECIMAL(10,3),
  piece_count                INT UNSIGNED,
  incoterm_code              CHAR(3),
  origin_locode              CHAR(5),
  destination_locode         CHAR(5),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (shipping_instruction_id),
  KEY idx_shipping_instruction_incoterm_code (incoterm_code),
  KEY fk_shipping_instruction_booking (booking_id),
  CONSTRAINT fk_shipping_instruction_booking FOREIGN KEY (booking_id)
    REFERENCES booking (booking_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Quotation, contract and booking of cargo: shipping instruction';

CREATE TABLE consignment (
  consignment_id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  booking_id                 BIGINT UNSIGNED NOT NULL,
  currency_code              CHAR(3),
  net_amount                 DECIMAL(14,2),
  discount_pct               DECIMAL(5,2),
  gross_weight_kg            DECIMAL(12,3),
  volume_cbm                 DECIMAL(10,3),
  piece_count                INT UNSIGNED,
  incoterm_code              CHAR(3),
  origin_locode              CHAR(5),
  destination_locode         CHAR(5),
  requested_pickup_on        DATETIME,
  promised_delivery_on       DATETIME,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (consignment_id),
  KEY idx_consignment_currency_code (currency_code),
  KEY fk_consignment_booking (booking_id),
  CONSTRAINT fk_consignment_booking FOREIGN KEY (booking_id)
    REFERENCES booking (booking_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE consignment_package (
  consignment_package_id     BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  origin_locode              CHAR(5),
  destination_locode         CHAR(5),
  requested_pickup_on        DATETIME,
  promised_delivery_on       DATETIME,
  special_instructions       TEXT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (consignment_package_id),
  KEY idx_consignment_package_origin_locode (origin_locode),
  KEY fk_consignment_package_consignment (consignment_id),
  CONSTRAINT fk_consignment_package_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE consignment_hazard (
  consignment_hazard_id      BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  discount_pct               DECIMAL(5,2),
  gross_weight_kg            DECIMAL(12,3),
  volume_cbm                 DECIMAL(10,3),
  piece_count                INT UNSIGNED,
  incoterm_code              CHAR(3),
  origin_locode              CHAR(5),
  destination_locode         CHAR(5),
  requested_pickup_on        DATETIME,
  promised_delivery_on       DATETIME,
  special_instructions       TEXT,
  discount_pct_rounded       DECIMAL(18,2) AS (ROUND(discount_pct, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (consignment_hazard_id),
  KEY idx_consignment_hazard_incoterm_code (incoterm_code),
  KEY idx_consignment_hazard_incoterm_code_created (incoterm_code, created_at),
  KEY fk_consignment_hazard_consignment (consignment_id),
  CONSTRAINT fk_consignment_hazard_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Quotation, contract and booking of cargo: consignment hazard';

CREATE TABLE consignment_temperature (
  consignment_temperature_id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  reference                  VARCHAR(32),
  status                     VARCHAR(24),
  issued_on                  DATE,
  expires_on                 DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (consignment_temperature_id),
  UNIQUE KEY ux_consignment_temperature_reference (reference),
  KEY fk_consignment_temperature_consignment (consignment_id),
  CONSTRAINT fk_consignment_temperature_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Quotation, contract and booking of cargo: consignment temperature';

CREATE TABLE consignment_milestone (
  consignment_milestone_id   BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  volume_cbm                 DECIMAL(10,3),
  piece_count                INT UNSIGNED,
  incoterm_code              CHAR(3),
  origin_locode              CHAR(5),
  destination_locode         CHAR(5),
  requested_pickup_on        DATETIME,
  promised_delivery_on       DATETIME,
  special_instructions       TEXT,
  sales_owner                VARCHAR(64),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (consignment_milestone_id),
  KEY idx_consignment_milestone_incoterm_code (incoterm_code),
  KEY fk_consignment_milestone_consignment (consignment_id),
  CONSTRAINT fk_consignment_milestone_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Quotation, contract and booking of cargo: consignment milestone';

CREATE TABLE transport_order (
  transport_order_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  job_reference              VARCHAR(32),
  status                     VARCHAR(24),
  planned_start_at           DATETIME,
  planned_end_at             DATETIME,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (transport_order_id),
  UNIQUE KEY ux_transport_order_job_reference (job_reference),
  KEY fk_transport_order_consignment (consignment_id),
  CONSTRAINT fk_transport_order_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE transport_leg (
  transport_leg_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  transport_order_id         BIGINT UNSIGNED NOT NULL,
  planned_end_at             DATETIME,
  actual_start_at            DATETIME,
  actual_end_at              DATETIME,
  distance_km                DECIMAL(9,2),
  sequence_no                SMALLINT UNSIGNED,
  mode_of_transport          VARCHAR(16),
  vehicle_registration       VARCHAR(24),
  seal_number                VARCHAR(24),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (transport_leg_id),
  KEY idx_transport_leg_mode_of_transport (mode_of_transport),
  KEY fk_transport_leg_transport_order (transport_order_id),
  CONSTRAINT fk_transport_leg_transport_order FOREIGN KEY (transport_order_id)
    REFERENCES transport_order (transport_order_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE leg_assignment (
  leg_assignment_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  transport_leg_id           BIGINT UNSIGNED NOT NULL,
  job_reference              VARCHAR(32),
  status                     VARCHAR(24),
  planned_start_at           DATETIME,
  planned_end_at             DATETIME,
  actual_start_at            DATETIME,
  actual_end_at              DATETIME,
  distance_km                DECIMAL(9,2),
  sequence_no                SMALLINT UNSIGNED,
  mode_of_transport          VARCHAR(16),
  vehicle_registration       VARCHAR(24),
  seal_number                VARCHAR(24),
  temperature_c              DECIMAL(5,2),
  event_code                 VARCHAR(16),
  distance_km_rounded        DECIMAL(18,2) AS (ROUND(distance_km, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (leg_assignment_id),
  UNIQUE KEY ux_leg_assignment_job_reference (job_reference),
  KEY fk_leg_assignment_transport_leg (transport_leg_id),
  CONSTRAINT fk_leg_assignment_transport_leg FOREIGN KEY (transport_leg_id)
    REFERENCES transport_leg (transport_leg_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Execution of a booking as physical movements: leg assignment';

CREATE TABLE drayage_job (
  drayage_job_id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  transport_order_id         BIGINT UNSIGNED NOT NULL,
  vehicle_registration       VARCHAR(24),
  seal_number                VARCHAR(24),
  temperature_c              DECIMAL(5,2),
  event_code                 VARCHAR(16),
  event_at                   DATETIME,
  location_locode            CHAR(5),
  delay_minutes              INT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (drayage_job_id),
  KEY idx_drayage_job_vehicle_registration (vehicle_registration),
  KEY fk_drayage_job_transport_order (transport_order_id),
  CONSTRAINT fk_drayage_job_transport_order FOREIGN KEY (transport_order_id)
    REFERENCES transport_order (transport_order_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Execution of a booking as physical movements: drayage job';

CREATE TABLE driver_assignment (
  driver_assignment_id       BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  drayage_job_id             BIGINT UNSIGNED NOT NULL,
  actual_end_at              DATETIME,
  distance_km                DECIMAL(9,2),
  sequence_no                SMALLINT UNSIGNED,
  mode_of_transport          VARCHAR(16),
  vehicle_registration       VARCHAR(24),
  seal_number                VARCHAR(24),
  temperature_c              DECIMAL(5,2),
  event_code                 VARCHAR(16),
  event_at                   DATETIME,
  location_locode            CHAR(5),
  delay_minutes              INT,
  operator_remarks           TEXT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (driver_assignment_id),
  KEY idx_driver_assignment_mode_of_transport (mode_of_transport),
  KEY fk_driver_assignment_drayage_job (drayage_job_id),
  CONSTRAINT fk_driver_assignment_drayage_job FOREIGN KEY (drayage_job_id)
    REFERENCES drayage_job (drayage_job_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Execution of a booking as physical movements: driver assignment';

CREATE TABLE gate_move (
  gate_move_id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  container_unit_id          BIGINT UNSIGNED NOT NULL,
  planned_start_at           DATETIME,
  planned_end_at             DATETIME,
  actual_start_at            DATETIME,
  actual_end_at              DATETIME,
  distance_km                DECIMAL(9,2),
  sequence_no                SMALLINT UNSIGNED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (gate_move_id),
  KEY idx_gate_move_distance_km (distance_km),
  KEY fk_gate_move_container_unit (container_unit_id),
  CONSTRAINT fk_gate_move_container_unit FOREIGN KEY (container_unit_id)
    REFERENCES container_unit (container_unit_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE yard_position (
  yard_position_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  warehouse_zone_id          BIGINT UNSIGNED NOT NULL,
  planned_start_at           DATETIME,
  planned_end_at             DATETIME,
  actual_start_at            DATETIME,
  actual_end_at              DATETIME,
  distance_km                DECIMAL(9,2),
  sequence_no                SMALLINT UNSIGNED,
  mode_of_transport          VARCHAR(16),
  vehicle_registration       VARCHAR(24),
  seal_number                VARCHAR(24),
  temperature_c              DECIMAL(5,2),
  event_code                 VARCHAR(16),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (yard_position_id),
  KEY idx_yard_position_mode_of_transport (mode_of_transport),
  KEY fk_yard_position_warehouse_zone (warehouse_zone_id),
  CONSTRAINT fk_yard_position_warehouse_zone FOREIGN KEY (warehouse_zone_id)
    REFERENCES warehouse_zone (warehouse_zone_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE stowage_plan (
  stowage_plan_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  voyage_id                  BIGINT UNSIGNED NOT NULL,
  sequence_no                SMALLINT UNSIGNED,
  mode_of_transport          VARCHAR(16),
  vehicle_registration       VARCHAR(24),
  seal_number                VARCHAR(24),
  temperature_c              DECIMAL(5,2),
  sequence_no_rounded        DECIMAL(18,2) AS (ROUND(sequence_no, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (stowage_plan_id),
  KEY idx_stowage_plan_mode_of_transport (mode_of_transport),
  KEY idx_stowage_plan_mode_of_transport_created (mode_of_transport, created_at),
  KEY fk_stowage_plan_voyage (voyage_id),
  CONSTRAINT fk_stowage_plan_voyage FOREIGN KEY (voyage_id)
    REFERENCES voyage (voyage_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Execution of a booking as physical movements: stowage plan';

CREATE TABLE stowage_slot (
  stowage_slot_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  stowage_plan_id            BIGINT UNSIGNED NOT NULL,
  distance_km                DECIMAL(9,2),
  sequence_no                SMALLINT UNSIGNED,
  mode_of_transport          VARCHAR(16),
  vehicle_registration       VARCHAR(24),
  seal_number                VARCHAR(24),
  temperature_c              DECIMAL(5,2),
  event_code                 VARCHAR(16),
  event_at                   DATETIME,
  location_locode            CHAR(5),
  delay_minutes              INT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (stowage_slot_id),
  KEY idx_stowage_slot_mode_of_transport (mode_of_transport),
  KEY fk_stowage_slot_stowage_plan (stowage_plan_id),
  CONSTRAINT fk_stowage_slot_stowage_plan FOREIGN KEY (stowage_plan_id)
    REFERENCES stowage_plan (stowage_plan_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Execution of a booking as physical movements: stowage slot';

CREATE TABLE loading_list (
  loading_list_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  port_call_id               BIGINT UNSIGNED NOT NULL,
  event_code                 VARCHAR(16),
  event_at                   DATETIME,
  location_locode            CHAR(5),
  delay_minutes              INT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (loading_list_id),
  KEY idx_loading_list_event_code (event_code),
  KEY fk_loading_list_port_call (port_call_id),
  CONSTRAINT fk_loading_list_port_call FOREIGN KEY (port_call_id)
    REFERENCES port_call (port_call_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Execution of a booking as physical movements: loading list';

CREATE TABLE discharge_list (
  discharge_list_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  port_call_id               BIGINT UNSIGNED NOT NULL,
  job_reference              VARCHAR(32),
  status                     VARCHAR(24),
  planned_start_at           DATETIME,
  planned_end_at             DATETIME,
  actual_start_at            DATETIME,
  actual_end_at              DATETIME,
  distance_km                DECIMAL(9,2),
  sequence_no                SMALLINT UNSIGNED,
  mode_of_transport          VARCHAR(16),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (discharge_list_id),
  UNIQUE KEY ux_discharge_list_job_reference (job_reference),
  KEY fk_discharge_list_port_call (port_call_id),
  CONSTRAINT fk_discharge_list_port_call FOREIGN KEY (port_call_id)
    REFERENCES port_call (port_call_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE transhipment (
  transhipment_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  planned_end_at             DATETIME,
  actual_start_at            DATETIME,
  actual_end_at              DATETIME,
  distance_km                DECIMAL(9,2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (transhipment_id),
  KEY idx_transhipment_distance_km (distance_km),
  KEY fk_transhipment_consignment (consignment_id),
  CONSTRAINT fk_transhipment_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE delivery_order (
  delivery_order_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  planned_end_at             DATETIME,
  actual_start_at            DATETIME,
  actual_end_at              DATETIME,
  distance_km                DECIMAL(9,2),
  sequence_no                SMALLINT UNSIGNED,
  mode_of_transport          VARCHAR(16),
  vehicle_registration       VARCHAR(24),
  seal_number                VARCHAR(24),
  distance_km_rounded        DECIMAL(18,2) AS (ROUND(distance_km, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (delivery_order_id),
  KEY idx_delivery_order_mode_of_transport (mode_of_transport),
  KEY fk_delivery_order_consignment (consignment_id),
  CONSTRAINT fk_delivery_order_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Execution of a booking as physical movements: delivery order';

CREATE TABLE proof_of_delivery (
  proof_of_delivery_id       BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  delivery_order_id          BIGINT UNSIGNED NOT NULL,
  planned_end_at             DATETIME,
  actual_start_at            DATETIME,
  actual_end_at              DATETIME,
  distance_km                DECIMAL(9,2),
  sequence_no                SMALLINT UNSIGNED,
  mode_of_transport          VARCHAR(16),
  vehicle_registration       VARCHAR(24),
  seal_number                VARCHAR(24),
  temperature_c              DECIMAL(5,2),
  event_code                 VARCHAR(16),
  event_at                   DATETIME,
  location_locode            CHAR(5),
  delay_minutes              INT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (proof_of_delivery_id),
  KEY idx_proof_of_delivery_mode_of_transport (mode_of_transport),
  KEY fk_proof_of_delivery_delivery_order (delivery_order_id),
  CONSTRAINT fk_proof_of_delivery_delivery_order FOREIGN KEY (delivery_order_id)
    REFERENCES delivery_order (delivery_order_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Execution of a booking as physical movements: proof of delivery';

CREATE TABLE tracking_event (
  tracking_event_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  distance_km                DECIMAL(9,2),
  sequence_no                SMALLINT UNSIGNED,
  mode_of_transport          VARCHAR(16),
  vehicle_registration       VARCHAR(24),
  seal_number                VARCHAR(24),
  temperature_c              DECIMAL(5,2),
  event_code                 VARCHAR(16),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (tracking_event_id),
  KEY idx_tracking_event_mode_of_transport (mode_of_transport),
  KEY fk_tracking_event_consignment (consignment_id),
  CONSTRAINT fk_tracking_event_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Execution of a booking as physical movements: tracking event';

CREATE TABLE warehouse_receipt (
  warehouse_receipt_id       BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  warehouse_site_id          BIGINT UNSIGNED NOT NULL,
  document_no                VARCHAR(32),
  status                     VARCHAR(24),
  received_on                DATE,
  despatched_on              DATE,
  quantity                   DECIMAL(14,3),
  quantity_reserved          DECIMAL(14,3),
  uom_code                   VARCHAR(8),
  batch_no                   VARCHAR(32),
  serial_no                  VARCHAR(48),
  expiry_date                DATE,
  location_code              VARCHAR(24),
  pallet_count               INT UNSIGNED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (warehouse_receipt_id),
  UNIQUE KEY ux_warehouse_receipt_document_no (document_no),
  KEY fk_warehouse_receipt_warehouse_site (warehouse_site_id),
  CONSTRAINT fk_warehouse_receipt_warehouse_site FOREIGN KEY (warehouse_site_id)
    REFERENCES warehouse_site (warehouse_site_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE receipt_line (
  receipt_line_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  warehouse_receipt_id       BIGINT UNSIGNED NOT NULL,
  despatched_on              DATE,
  quantity                   DECIMAL(14,3),
  quantity_reserved          DECIMAL(14,3),
  uom_code                   VARCHAR(8),
  batch_no                   VARCHAR(32),
  serial_no                  VARCHAR(48),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (receipt_line_id),
  KEY idx_receipt_line_uom_code (uom_code),
  KEY fk_receipt_line_warehouse_receipt (warehouse_receipt_id),
  CONSTRAINT fk_receipt_line_warehouse_receipt FOREIGN KEY (warehouse_receipt_id)
    REFERENCES warehouse_receipt (warehouse_receipt_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE putaway_task (
  putaway_task_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  receipt_line_id            BIGINT UNSIGNED NOT NULL,
  uom_code                   VARCHAR(8),
  batch_no                   VARCHAR(32),
  serial_no                  VARCHAR(48),
  expiry_date                DATE,
  location_code              VARCHAR(24),
  pallet_count               INT UNSIGNED,
  net_weight_kg              DECIMAL(12,3),
  storage_rate               DECIMAL(10,4),
  handling_rate              DECIMAL(10,4),
  operator_code              VARCHAR(24),
  stock_notes                TEXT,
  pallet_count_rounded       DECIMAL(18,2) AS (ROUND(pallet_count, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (putaway_task_id),
  KEY idx_putaway_task_uom_code (uom_code),
  KEY idx_putaway_task_uom_code_created (uom_code, created_at),
  KEY fk_putaway_task_receipt_line (receipt_line_id),
  CONSTRAINT fk_putaway_task_receipt_line FOREIGN KEY (receipt_line_id)
    REFERENCES receipt_line (receipt_line_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Warehouse receipt, storage and despatch: putaway task';

CREATE TABLE stock_item (
  stock_item_id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  warehouse_zone_id          BIGINT UNSIGNED NOT NULL,
  expiry_date                DATE,
  location_code              VARCHAR(24),
  pallet_count               INT UNSIGNED,
  net_weight_kg              DECIMAL(12,3),
  storage_rate               DECIMAL(10,4),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (stock_item_id),
  KEY idx_stock_item_location_code (location_code),
  KEY fk_stock_item_warehouse_zone (warehouse_zone_id),
  CONSTRAINT fk_stock_item_warehouse_zone FOREIGN KEY (warehouse_zone_id)
    REFERENCES warehouse_zone (warehouse_zone_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Warehouse receipt, storage and despatch: stock item';

CREATE TABLE stock_movement (
  stock_movement_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  stock_item_id              BIGINT UNSIGNED NOT NULL,
  despatched_on              DATE,
  quantity                   DECIMAL(14,3),
  quantity_reserved          DECIMAL(14,3),
  uom_code                   VARCHAR(8),
  batch_no                   VARCHAR(32),
  serial_no                  VARCHAR(48),
  expiry_date                DATE,
  location_code              VARCHAR(24),
  pallet_count               INT UNSIGNED,
  net_weight_kg              DECIMAL(12,3),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (stock_movement_id),
  KEY idx_stock_movement_uom_code (uom_code),
  KEY fk_stock_movement_stock_item (stock_item_id),
  CONSTRAINT fk_stock_movement_stock_item FOREIGN KEY (stock_item_id)
    REFERENCES stock_item (stock_item_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Warehouse receipt, storage and despatch: stock movement';

CREATE TABLE stock_count (
  stock_count_id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  warehouse_zone_id          BIGINT UNSIGNED NOT NULL,
  document_no                VARCHAR(32),
  status                     VARCHAR(24),
  received_on                DATE,
  despatched_on              DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (stock_count_id),
  UNIQUE KEY ux_stock_count_document_no (document_no),
  KEY fk_stock_count_warehouse_zone (warehouse_zone_id),
  CONSTRAINT fk_stock_count_warehouse_zone FOREIGN KEY (warehouse_zone_id)
    REFERENCES warehouse_zone (warehouse_zone_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE count_line (
  count_line_id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  stock_count_id             BIGINT UNSIGNED NOT NULL,
  serial_no                  VARCHAR(48),
  expiry_date                DATE,
  location_code              VARCHAR(24),
  pallet_count               INT UNSIGNED,
  net_weight_kg              DECIMAL(12,3),
  storage_rate               DECIMAL(10,4),
  handling_rate              DECIMAL(10,4),
  operator_code              VARCHAR(24),
  stock_notes                TEXT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (count_line_id),
  UNIQUE KEY ux_count_line_serial_no (serial_no),
  KEY fk_count_line_stock_count (stock_count_id),
  CONSTRAINT fk_count_line_stock_count FOREIGN KEY (stock_count_id)
    REFERENCES stock_count (stock_count_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE pick_order (
  pick_order_id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  warehouse_site_id          BIGINT UNSIGNED NOT NULL,
  uom_code                   VARCHAR(8),
  batch_no                   VARCHAR(32),
  serial_no                  VARCHAR(48),
  expiry_date                DATE,
  expiry_date_year           SMALLINT UNSIGNED AS (YEAR(expiry_date)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (pick_order_id),
  KEY idx_pick_order_uom_code (uom_code),
  KEY fk_pick_order_warehouse_site (warehouse_site_id),
  CONSTRAINT fk_pick_order_warehouse_site FOREIGN KEY (warehouse_site_id)
    REFERENCES warehouse_site (warehouse_site_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Warehouse receipt, storage and despatch: pick order';

CREATE TABLE pick_line (
  pick_line_id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  pick_order_id              BIGINT UNSIGNED NOT NULL,
  received_on                DATE,
  despatched_on              DATE,
  quantity                   DECIMAL(14,3),
  quantity_reserved          DECIMAL(14,3),
  uom_code                   VARCHAR(8),
  batch_no                   VARCHAR(32),
  serial_no                  VARCHAR(48),
  expiry_date                DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (pick_line_id),
  KEY idx_pick_line_uom_code (uom_code),
  KEY fk_pick_line_pick_order (pick_order_id),
  CONSTRAINT fk_pick_line_pick_order FOREIGN KEY (pick_order_id)
    REFERENCES pick_order (pick_order_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Warehouse receipt, storage and despatch: pick line';

CREATE TABLE packing_task (
  packing_task_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  pick_order_id              BIGINT UNSIGNED NOT NULL,
  despatched_on              DATE,
  quantity                   DECIMAL(14,3),
  quantity_reserved          DECIMAL(14,3),
  uom_code                   VARCHAR(8),
  batch_no                   VARCHAR(32),
  serial_no                  VARCHAR(48),
  expiry_date                DATE,
  location_code              VARCHAR(24),
  pallet_count               INT UNSIGNED,
  net_weight_kg              DECIMAL(12,3),
  storage_rate               DECIMAL(10,4),
  handling_rate              DECIMAL(10,4),
  operator_code              VARCHAR(24),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (packing_task_id),
  KEY idx_packing_task_uom_code (uom_code),
  KEY fk_packing_task_pick_order (pick_order_id),
  CONSTRAINT fk_packing_task_pick_order FOREIGN KEY (pick_order_id)
    REFERENCES pick_order (pick_order_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Warehouse receipt, storage and despatch: packing task';

CREATE TABLE despatch_note (
  despatch_note_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  pick_order_id              BIGINT UNSIGNED NOT NULL,
  uom_code                   VARCHAR(8),
  batch_no                   VARCHAR(32),
  serial_no                  VARCHAR(48),
  expiry_date                DATE,
  location_code              VARCHAR(24),
  pallet_count               INT UNSIGNED,
  net_weight_kg              DECIMAL(12,3),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (despatch_note_id),
  KEY idx_despatch_note_uom_code (uom_code),
  KEY fk_despatch_note_pick_order (pick_order_id),
  CONSTRAINT fk_despatch_note_pick_order FOREIGN KEY (pick_order_id)
    REFERENCES pick_order (pick_order_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE despatch_line (
  despatch_line_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  despatch_note_id           BIGINT UNSIGNED NOT NULL,
  quantity_reserved          DECIMAL(14,3),
  uom_code                   VARCHAR(8),
  batch_no                   VARCHAR(32),
  serial_no                  VARCHAR(48),
  expiry_date                DATE,
  location_code              VARCHAR(24),
  pallet_count               INT UNSIGNED,
  net_weight_kg              DECIMAL(12,3),
  storage_rate               DECIMAL(10,4),
  handling_rate              DECIMAL(10,4),
  operator_code              VARCHAR(24),
  stock_notes                TEXT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (despatch_line_id),
  KEY idx_despatch_line_uom_code (uom_code),
  KEY fk_despatch_line_despatch_note (despatch_note_id),
  CONSTRAINT fk_despatch_line_despatch_note FOREIGN KEY (despatch_note_id)
    REFERENCES despatch_note (despatch_note_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE storage_charge (
  storage_charge_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  stock_item_id              BIGINT UNSIGNED NOT NULL,
  location_code              VARCHAR(24),
  pallet_count               INT UNSIGNED,
  net_weight_kg              DECIMAL(12,3),
  storage_rate               DECIMAL(10,4),
  handling_rate              DECIMAL(10,4),
  operator_code              VARCHAR(24),
  pallet_count_rounded       DECIMAL(18,2) AS (ROUND(pallet_count, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (storage_charge_id),
  KEY idx_storage_charge_location_code (location_code),
  KEY idx_storage_charge_location_code_created (location_code, created_at),
  KEY fk_storage_charge_stock_item (stock_item_id),
  CONSTRAINT fk_storage_charge_stock_item FOREIGN KEY (stock_item_id)
    REFERENCES stock_item (stock_item_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Warehouse receipt, storage and despatch: storage charge';

CREATE TABLE handling_task (
  handling_task_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  warehouse_site_id          BIGINT UNSIGNED NOT NULL,
  batch_no                   VARCHAR(32),
  serial_no                  VARCHAR(48),
  expiry_date                DATE,
  location_code              VARCHAR(24),
  pallet_count               INT UNSIGNED,
  net_weight_kg              DECIMAL(12,3),
  storage_rate               DECIMAL(10,4),
  handling_rate              DECIMAL(10,4),
  operator_code              VARCHAR(24),
  stock_notes                TEXT,
  label_payload              JSON,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (handling_task_id),
  KEY idx_handling_task_batch_no (batch_no),
  KEY fk_handling_task_warehouse_site (warehouse_site_id),
  CONSTRAINT fk_handling_task_warehouse_site FOREIGN KEY (warehouse_site_id)
    REFERENCES warehouse_site (warehouse_site_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Warehouse receipt, storage and despatch: handling task';

CREATE TABLE customs_declaration (
  customs_declaration_id     BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  declaration_no             VARCHAR(32),
  regime_code                VARCHAR(8),
  status                     VARCHAR(24),
  lodged_on                  DATETIME,
  cleared_on                 DATETIME,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (customs_declaration_id),
  UNIQUE KEY ux_customs_declaration_declaration_no (declaration_no),
  KEY fk_customs_declaration_consignment (consignment_id),
  CONSTRAINT fk_customs_declaration_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Customs, duty and regulatory compliance: customs declaration';

CREATE TABLE declaration_line (
  declaration_line_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  customs_declaration_id     BIGINT UNSIGNED NOT NULL,
  lodged_on                  DATETIME,
  cleared_on                 DATETIME,
  commodity_code             VARCHAR(12),
  country_of_origin          CHAR(2),
  statistical_value          DECIMAL(14,2),
  duty_amount                DECIMAL(14,2),
  vat_amount                 DECIMAL(14,2),
  net_mass_kg                DECIMAL(12,3),
  supplementary_units        DECIMAL(12,3),
  guarantee_reference        VARCHAR(48),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (declaration_line_id),
  KEY idx_declaration_line_commodity_code (commodity_code),
  KEY fk_declaration_line_customs_declaration (customs_declaration_id),
  CONSTRAINT fk_declaration_line_customs_declaration FOREIGN KEY (customs_declaration_id)
    REFERENCES customs_declaration (customs_declaration_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE declaration_document (
  declaration_document_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  customs_declaration_id     BIGINT UNSIGNED NOT NULL,
  country_of_origin          CHAR(2),
  statistical_value          DECIMAL(14,2),
  duty_amount                DECIMAL(14,2),
  vat_amount                 DECIMAL(14,2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (declaration_document_id),
  KEY idx_declaration_document_country_of_origin (country_of_origin),
  KEY fk_declaration_document_customs_declaration (customs_declaration_id),
  CONSTRAINT fk_declaration_document_customs_declaration FOREIGN KEY (customs_declaration_id)
    REFERENCES customs_declaration (customs_declaration_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE tariff_classification (
  tariff_classification_id   BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  hs_chapter_id              BIGINT UNSIGNED NOT NULL,
  vat_amount                 DECIMAL(14,2),
  net_mass_kg                DECIMAL(12,3),
  supplementary_units        DECIMAL(12,3),
  guarantee_reference        VARCHAR(48),
  office_code                VARCHAR(12),
  inspection_required        TINYINT(1) NOT NULL DEFAULT 0,
  risk_score                 SMALLINT UNSIGNED,
  officer_notes              TEXT,
  declaration_payload        JSON,
  vat_amount_rounded         DECIMAL(18,2) AS (ROUND(vat_amount, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (tariff_classification_id),
  KEY idx_tariff_classification_guarantee_reference (guarantee_reference),
  KEY fk_tariff_classification_hs_chapter (hs_chapter_id),
  CONSTRAINT fk_tariff_classification_hs_chapter FOREIGN KEY (hs_chapter_id)
    REFERENCES hs_chapter (hs_chapter_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Customs, duty and regulatory compliance: tariff classification';

CREATE TABLE duty_calculation (
  duty_calculation_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  declaration_line_id        BIGINT UNSIGNED NOT NULL,
  guarantee_reference        VARCHAR(48),
  office_code                VARCHAR(12),
  inspection_required        TINYINT(1) NOT NULL DEFAULT 0,
  risk_score                 SMALLINT UNSIGNED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (duty_calculation_id),
  KEY idx_duty_calculation_guarantee_reference (guarantee_reference),
  KEY fk_duty_calculation_declaration_line (declaration_line_id),
  CONSTRAINT fk_duty_calculation_declaration_line FOREIGN KEY (declaration_line_id)
    REFERENCES declaration_line (declaration_line_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Customs, duty and regulatory compliance: duty calculation';

CREATE TABLE duty_payment (
  duty_payment_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  duty_calculation_id        BIGINT UNSIGNED NOT NULL,
  cleared_on                 DATETIME,
  commodity_code             VARCHAR(12),
  country_of_origin          CHAR(2),
  statistical_value          DECIMAL(14,2),
  duty_amount                DECIMAL(14,2),
  vat_amount                 DECIMAL(14,2),
  net_mass_kg                DECIMAL(12,3),
  supplementary_units        DECIMAL(12,3),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (duty_payment_id),
  KEY idx_duty_payment_commodity_code (commodity_code),
  KEY fk_duty_payment_duty_calculation (duty_calculation_id),
  CONSTRAINT fk_duty_payment_duty_calculation FOREIGN KEY (duty_calculation_id)
    REFERENCES duty_calculation (duty_calculation_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Customs, duty and regulatory compliance: duty payment';

CREATE TABLE customs_status (
  customs_status_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  customs_declaration_id     BIGINT UNSIGNED NOT NULL,
  declaration_no             VARCHAR(32),
  regime_code                VARCHAR(8),
  status                     VARCHAR(24),
  lodged_on                  DATETIME,
  cleared_on                 DATETIME,
  commodity_code             VARCHAR(12),
  country_of_origin          CHAR(2),
  statistical_value          DECIMAL(14,2),
  duty_amount                DECIMAL(14,2),
  vat_amount                 DECIMAL(14,2),
  net_mass_kg                DECIMAL(12,3),
  supplementary_units        DECIMAL(12,3),
  guarantee_reference        VARCHAR(48),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (customs_status_id),
  UNIQUE KEY ux_customs_status_declaration_no (declaration_no),
  KEY fk_customs_status_customs_declaration (customs_declaration_id),
  CONSTRAINT fk_customs_status_customs_declaration FOREIGN KEY (customs_declaration_id)
    REFERENCES customs_declaration (customs_declaration_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE transit_movement (
  transit_movement_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  vat_amount                 DECIMAL(14,2),
  net_mass_kg                DECIMAL(12,3),
  supplementary_units        DECIMAL(12,3),
  guarantee_reference        VARCHAR(48),
  office_code                VARCHAR(12),
  inspection_required        TINYINT(1) NOT NULL DEFAULT 0,
  risk_score                 SMALLINT UNSIGNED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (transit_movement_id),
  KEY idx_transit_movement_guarantee_reference (guarantee_reference),
  KEY fk_transit_movement_consignment (consignment_id),
  CONSTRAINT fk_transit_movement_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE transit_guarantee (
  transit_guarantee_id       BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  transit_movement_id        BIGINT UNSIGNED NOT NULL,
  lodged_on                  DATETIME,
  cleared_on                 DATETIME,
  commodity_code             VARCHAR(12),
  country_of_origin          CHAR(2),
  statistical_value          DECIMAL(14,2),
  duty_amount                DECIMAL(14,2),
  vat_amount                 DECIMAL(14,2),
  net_mass_kg                DECIMAL(12,3),
  supplementary_units        DECIMAL(12,3),
  guarantee_reference        VARCHAR(48),
  office_code                VARCHAR(12),
  inspection_required        TINYINT(1) NOT NULL DEFAULT 0,
  statistical_value_rounded  DECIMAL(18,2) AS (ROUND(statistical_value, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (transit_guarantee_id),
  KEY idx_transit_guarantee_commodity_code (commodity_code),
  KEY idx_transit_guarantee_commodity_code_created (commodity_code, created_at),
  KEY fk_transit_guarantee_transit_movement (transit_movement_id),
  CONSTRAINT fk_transit_guarantee_transit_movement FOREIGN KEY (transit_movement_id)
    REFERENCES transit_movement (transit_movement_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Customs, duty and regulatory compliance: transit guarantee';

CREATE TABLE sanctions_screening (
  sanctions_screening_id     BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  regime_code                VARCHAR(8),
  status                     VARCHAR(24),
  lodged_on                  DATETIME,
  cleared_on                 DATETIME,
  commodity_code             VARCHAR(12),
  country_of_origin          CHAR(2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (sanctions_screening_id),
  KEY idx_sanctions_screening_regime_code (regime_code),
  KEY fk_sanctions_screening_organisation (organisation_id),
  CONSTRAINT fk_sanctions_screening_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Customs, duty and regulatory compliance: sanctions screening';

CREATE TABLE screening_hit (
  screening_hit_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  sanctions_screening_id     BIGINT UNSIGNED NOT NULL,
  country_of_origin          CHAR(2),
  statistical_value          DECIMAL(14,2),
  duty_amount                DECIMAL(14,2),
  vat_amount                 DECIMAL(14,2),
  net_mass_kg                DECIMAL(12,3),
  supplementary_units        DECIMAL(12,3),
  guarantee_reference        VARCHAR(48),
  office_code                VARCHAR(12),
  inspection_required        TINYINT(1) NOT NULL DEFAULT 0,
  risk_score                 SMALLINT UNSIGNED,
  officer_notes              TEXT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (screening_hit_id),
  KEY idx_screening_hit_country_of_origin (country_of_origin),
  KEY fk_screening_hit_sanctions_screening (sanctions_screening_id),
  CONSTRAINT fk_screening_hit_sanctions_screening FOREIGN KEY (sanctions_screening_id)
    REFERENCES sanctions_screening (sanctions_screening_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Customs, duty and regulatory compliance: screening hit';

CREATE TABLE dangerous_goods_declaration (
  dangerous_goods_declaration_id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  commodity_code             VARCHAR(12),
  country_of_origin          CHAR(2),
  statistical_value          DECIMAL(14,2),
  duty_amount                DECIMAL(14,2),
  vat_amount                 DECIMAL(14,2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (dangerous_goods_declaration_id),
  KEY idx_dangerous_goods_declaration_commodity_code (commodity_code),
  KEY fk_dangerous_goods_declaration_consignment (consignment_id),
  CONSTRAINT fk_dangerous_goods_declaration_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE dg_segregation_check (
  dg_segregation_check_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  dangerous_goods_declaration_id BIGINT UNSIGNED NOT NULL,
  declaration_no             VARCHAR(32),
  regime_code                VARCHAR(8),
  status                     VARCHAR(24),
  lodged_on                  DATETIME,
  cleared_on                 DATETIME,
  commodity_code             VARCHAR(12),
  country_of_origin          CHAR(2),
  statistical_value          DECIMAL(14,2),
  duty_amount                DECIMAL(14,2),
  vat_amount                 DECIMAL(14,2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (dg_segregation_check_id),
  UNIQUE KEY ux_dg_segregation_check_declaration_no (declaration_no),
  KEY fk_dg_segregation_check_dangerous_goods_declaration (dangerous_goods_declaration_id),
  CONSTRAINT fk_dg_segregation_check_dangerous_goods_declaration FOREIGN KEY (dangerous_goods_declaration_id)
    REFERENCES dangerous_goods_declaration (dangerous_goods_declaration_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE phytosanitary_certificate (
  phytosanitary_certificate_id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  vat_amount                 DECIMAL(14,2),
  net_mass_kg                DECIMAL(12,3),
  supplementary_units        DECIMAL(12,3),
  guarantee_reference        VARCHAR(48),
  vat_amount_rounded         DECIMAL(18,2) AS (ROUND(vat_amount, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (phytosanitary_certificate_id),
  KEY idx_phytosanitary_certificate_guarantee_reference (guarantee_reference),
  KEY fk_phytosanitary_certificate_consignment (consignment_id),
  CONSTRAINT fk_phytosanitary_certificate_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Customs, duty and regulatory compliance: phytosanitary certificate';

CREATE TABLE export_licence (
  export_licence_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  status                     VARCHAR(24),
  lodged_on                  DATETIME,
  cleared_on                 DATETIME,
  commodity_code             VARCHAR(12),
  country_of_origin          CHAR(2),
  statistical_value          DECIMAL(14,2),
  duty_amount                DECIMAL(14,2),
  vat_amount                 DECIMAL(14,2),
  net_mass_kg                DECIMAL(12,3),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (export_licence_id),
  KEY idx_export_licence_status (status),
  KEY fk_export_licence_consignment (consignment_id),
  CONSTRAINT fk_export_licence_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Customs, duty and regulatory compliance: export licence';

CREATE TABLE charge (
  charge_id                  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  document_no                VARCHAR(32),
  status                     VARCHAR(24),
  issued_on                  DATE,
  due_on                     DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (charge_id),
  UNIQUE KEY ux_charge_document_no (document_no),
  KEY fk_charge_consignment (consignment_id),
  CONSTRAINT fk_charge_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Billing, settlement and the general ledger: charge';

CREATE TABLE charge_allocation (
  charge_allocation_id       BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  charge_id                  BIGINT UNSIGNED NOT NULL,
  due_on                     DATE,
  currency_code              CHAR(3),
  fx_rate                    DECIMAL(14,6),
  net_amount                 DECIMAL(14,2),
  tax_amount                 DECIMAL(14,2),
  gross_amount               DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  cost_centre                VARCHAR(16),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (charge_allocation_id),
  KEY idx_charge_allocation_currency_code (currency_code),
  KEY fk_charge_allocation_charge (charge_id),
  CONSTRAINT fk_charge_allocation_charge FOREIGN KEY (charge_id)
    REFERENCES charge (charge_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE invoice (
  invoice_id                 BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  customer_id                BIGINT UNSIGNED NOT NULL,
  document_no                VARCHAR(32),
  status                     VARCHAR(24),
  issued_on                  DATE,
  due_on                     DATE,
  currency_code              CHAR(3),
  fx_rate                    DECIMAL(14,6),
  net_amount                 DECIMAL(14,2),
  tax_amount                 DECIMAL(14,2),
  gross_amount               DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  cost_centre                VARCHAR(16),
  ledger_account             VARCHAR(16),
  posting_period             CHAR(7),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (invoice_id),
  UNIQUE KEY ux_invoice_document_no (document_no),
  KEY fk_invoice_customer (customer_id),
  CONSTRAINT fk_invoice_customer FOREIGN KEY (customer_id)
    REFERENCES customer (customer_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE invoice_line (
  invoice_line_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  invoice_id                 BIGINT UNSIGNED NOT NULL,
  settled_amount             DECIMAL(14,2),
  cost_centre                VARCHAR(16),
  ledger_account             VARCHAR(16),
  posting_period             CHAR(7),
  posted_at                  DATETIME,
  reversal_of                VARCHAR(32),
  approved_by                VARCHAR(64),
  settled_amount_rounded     DECIMAL(18,2) AS (ROUND(settled_amount, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (invoice_line_id),
  KEY idx_invoice_line_cost_centre (cost_centre),
  KEY idx_invoice_line_cost_centre_created (cost_centre, created_at),
  KEY fk_invoice_line_invoice (invoice_id),
  CONSTRAINT fk_invoice_line_invoice FOREIGN KEY (invoice_id)
    REFERENCES invoice (invoice_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Billing, settlement and the general ledger: invoice line';

CREATE TABLE credit_note (
  credit_note_id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  invoice_id                 BIGINT UNSIGNED NOT NULL,
  fx_rate                    DECIMAL(14,6),
  net_amount                 DECIMAL(14,2),
  tax_amount                 DECIMAL(14,2),
  gross_amount               DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  cost_centre                VARCHAR(16),
  ledger_account             VARCHAR(16),
  posting_period             CHAR(7),
  posted_at                  DATETIME,
  reversal_of                VARCHAR(32),
  approved_by                VARCHAR(64),
  narrative                  TEXT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (credit_note_id),
  KEY idx_credit_note_cost_centre (cost_centre),
  KEY fk_credit_note_invoice (invoice_id),
  CONSTRAINT fk_credit_note_invoice FOREIGN KEY (invoice_id)
    REFERENCES invoice (invoice_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Billing, settlement and the general ledger: credit note';

CREATE TABLE credit_note_line (
  credit_note_line_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  credit_note_id             BIGINT UNSIGNED NOT NULL,
  issued_on                  DATE,
  due_on                     DATE,
  currency_code              CHAR(3),
  fx_rate                    DECIMAL(14,6),
  net_amount                 DECIMAL(14,2),
  tax_amount                 DECIMAL(14,2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (credit_note_line_id),
  KEY idx_credit_note_line_currency_code (currency_code),
  KEY fk_credit_note_line_credit_note (credit_note_id),
  CONSTRAINT fk_credit_note_line_credit_note FOREIGN KEY (credit_note_id)
    REFERENCES credit_note (credit_note_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Billing, settlement and the general ledger: credit note line';

CREATE TABLE payment (
  payment_id                 BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  invoice_id                 BIGINT UNSIGNED NOT NULL,
  issued_on                  DATE,
  due_on                     DATE,
  currency_code              CHAR(3),
  fx_rate                    DECIMAL(14,6),
  net_amount                 DECIMAL(14,2),
  tax_amount                 DECIMAL(14,2),
  gross_amount               DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  cost_centre                VARCHAR(16),
  ledger_account             VARCHAR(16),
  posting_period             CHAR(7),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (payment_id),
  KEY idx_payment_currency_code (currency_code),
  KEY fk_payment_invoice (invoice_id),
  CONSTRAINT fk_payment_invoice FOREIGN KEY (invoice_id)
    REFERENCES invoice (invoice_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE payment_allocation (
  payment_allocation_id      BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  payment_id                 BIGINT UNSIGNED NOT NULL,
  tax_amount                 DECIMAL(14,2),
  gross_amount               DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  cost_centre                VARCHAR(16),
  ledger_account             VARCHAR(16),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (payment_allocation_id),
  KEY idx_payment_allocation_cost_centre (cost_centre),
  KEY fk_payment_allocation_payment (payment_id),
  CONSTRAINT fk_payment_allocation_payment FOREIGN KEY (payment_id)
    REFERENCES payment (payment_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE cash_receipt (
  cash_receipt_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  customer_id                BIGINT UNSIGNED NOT NULL,
  net_amount                 DECIMAL(14,2),
  tax_amount                 DECIMAL(14,2),
  gross_amount               DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  cost_centre                VARCHAR(16),
  ledger_account             VARCHAR(16),
  posting_period             CHAR(7),
  posted_at                  DATETIME,
  reversal_of                VARCHAR(32),
  approved_by                VARCHAR(64),
  net_amount_rounded         DECIMAL(18,2) AS (ROUND(net_amount, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (cash_receipt_id),
  KEY idx_cash_receipt_cost_centre (cost_centre),
  KEY fk_cash_receipt_customer (customer_id),
  CONSTRAINT fk_cash_receipt_customer FOREIGN KEY (customer_id)
    REFERENCES customer (customer_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Billing, settlement and the general ledger: cash receipt';

CREATE TABLE accrual (
  accrual_id                 BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  posting_period             CHAR(7),
  posted_at                  DATETIME,
  reversal_of                VARCHAR(32),
  approved_by                VARCHAR(64),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (accrual_id),
  KEY idx_accrual_posting_period (posting_period),
  KEY fk_accrual_consignment (consignment_id),
  CONSTRAINT fk_accrual_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Billing, settlement and the general ledger: accrual';

CREATE TABLE cost_estimate (
  cost_estimate_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  document_no                VARCHAR(32),
  status                     VARCHAR(24),
  issued_on                  DATE,
  due_on                     DATE,
  currency_code              CHAR(3),
  fx_rate                    DECIMAL(14,6),
  net_amount                 DECIMAL(14,2),
  tax_amount                 DECIMAL(14,2),
  gross_amount               DECIMAL(14,2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (cost_estimate_id),
  UNIQUE KEY ux_cost_estimate_document_no (document_no),
  KEY fk_cost_estimate_consignment (consignment_id),
  CONSTRAINT fk_cost_estimate_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Billing, settlement and the general ledger: cost estimate';

CREATE TABLE cost_actual (
  cost_actual_id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  cost_estimate_id           BIGINT UNSIGNED NOT NULL,
  due_on                     DATE,
  currency_code              CHAR(3),
  fx_rate                    DECIMAL(14,6),
  net_amount                 DECIMAL(14,2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (cost_actual_id),
  KEY idx_cost_actual_currency_code (currency_code),
  KEY fk_cost_actual_cost_estimate (cost_estimate_id),
  CONSTRAINT fk_cost_actual_cost_estimate FOREIGN KEY (cost_estimate_id)
    REFERENCES cost_estimate (cost_estimate_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE purchase_invoice (
  purchase_invoice_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vendor_id                  BIGINT UNSIGNED NOT NULL,
  due_on                     DATE,
  currency_code              CHAR(3),
  fx_rate                    DECIMAL(14,6),
  net_amount                 DECIMAL(14,2),
  tax_amount                 DECIMAL(14,2),
  gross_amount               DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  cost_centre                VARCHAR(16),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (purchase_invoice_id),
  KEY idx_purchase_invoice_currency_code (currency_code),
  KEY fk_purchase_invoice_vendor (vendor_id),
  CONSTRAINT fk_purchase_invoice_vendor FOREIGN KEY (vendor_id)
    REFERENCES vendor (vendor_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE purchase_invoice_line (
  purchase_invoice_line_id   BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  purchase_invoice_id        BIGINT UNSIGNED NOT NULL,
  due_on                     DATE,
  currency_code              CHAR(3),
  fx_rate                    DECIMAL(14,6),
  net_amount                 DECIMAL(14,2),
  tax_amount                 DECIMAL(14,2),
  gross_amount               DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  cost_centre                VARCHAR(16),
  ledger_account             VARCHAR(16),
  posting_period             CHAR(7),
  posted_at                  DATETIME,
  reversal_of                VARCHAR(32),
  approved_by                VARCHAR(64),
  fx_rate_rounded            DECIMAL(18,2) AS (ROUND(fx_rate, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (purchase_invoice_line_id),
  KEY idx_purchase_invoice_line_currency_code (currency_code),
  KEY idx_purchase_invoice_line_currency_code_created (currency_code, created_at),
  KEY fk_purchase_invoice_line_purchase_invoice (purchase_invoice_id),
  CONSTRAINT fk_purchase_invoice_line_purchase_invoice FOREIGN KEY (purchase_invoice_id)
    REFERENCES purchase_invoice (purchase_invoice_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Billing, settlement and the general ledger: purchase invoice line';

CREATE TABLE disbursement (
  disbursement_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  voyage_id                  BIGINT UNSIGNED NOT NULL,
  net_amount                 DECIMAL(14,2),
  tax_amount                 DECIMAL(14,2),
  gross_amount               DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  cost_centre                VARCHAR(16),
  ledger_account             VARCHAR(16),
  posting_period             CHAR(7),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (disbursement_id),
  KEY idx_disbursement_cost_centre (cost_centre),
  KEY fk_disbursement_voyage (voyage_id),
  CONSTRAINT fk_disbursement_voyage FOREIGN KEY (voyage_id)
    REFERENCES voyage (voyage_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Billing, settlement and the general ledger: disbursement';

CREATE TABLE general_ledger_entry (
  general_ledger_entry_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  charge_code_id             BIGINT UNSIGNED NOT NULL,
  due_on                     DATE,
  currency_code              CHAR(3),
  fx_rate                    DECIMAL(14,6),
  net_amount                 DECIMAL(14,2),
  tax_amount                 DECIMAL(14,2),
  gross_amount               DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  cost_centre                VARCHAR(16),
  ledger_account             VARCHAR(16),
  posting_period             CHAR(7),
  posted_at                  DATETIME,
  reversal_of                VARCHAR(32),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (general_ledger_entry_id),
  KEY idx_general_ledger_entry_currency_code (currency_code),
  KEY fk_general_ledger_entry_charge_code (charge_code_id),
  CONSTRAINT fk_general_ledger_entry_charge_code FOREIGN KEY (charge_code_id)
    REFERENCES charge_code (charge_code_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Billing, settlement and the general ledger: general ledger entry';

CREATE TABLE tax_line (
  tax_line_id                BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  invoice_line_id            BIGINT UNSIGNED NOT NULL,
  settled_amount             DECIMAL(14,2),
  cost_centre                VARCHAR(16),
  ledger_account             VARCHAR(16),
  posting_period             CHAR(7),
  posted_at                  DATETIME,
  reversal_of                VARCHAR(32),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (tax_line_id),
  KEY idx_tax_line_cost_centre (cost_centre),
  KEY fk_tax_line_invoice_line (invoice_line_id),
  CONSTRAINT fk_tax_line_invoice_line FOREIGN KEY (invoice_line_id)
    REFERENCES invoice_line (invoice_line_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE exchange_rate (
  exchange_rate_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  currency_id                BIGINT UNSIGNED NOT NULL,
  due_on                     DATE,
  currency_code              CHAR(3),
  fx_rate                    DECIMAL(14,6),
  net_amount                 DECIMAL(14,2),
  tax_amount                 DECIMAL(14,2),
  gross_amount               DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  cost_centre                VARCHAR(16),
  ledger_account             VARCHAR(16),
  posting_period             CHAR(7),
  posted_at                  DATETIME,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (exchange_rate_id),
  KEY idx_exchange_rate_currency_code (currency_code),
  KEY fk_exchange_rate_currency (currency_id),
  CONSTRAINT fk_exchange_rate_currency FOREIGN KEY (currency_id)
    REFERENCES currency (currency_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE purchase_requisition (
  purchase_requisition_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vendor_id                  BIGINT UNSIGNED NOT NULL,
  document_no                VARCHAR(32),
  status                     VARCHAR(24),
  raised_on                  DATE,
  required_by                DATE,
  currency_code              CHAR(3),
  raised_on_year             SMALLINT UNSIGNED AS (YEAR(raised_on)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (purchase_requisition_id),
  UNIQUE KEY ux_purchase_requisition_document_no (document_no),
  KEY fk_purchase_requisition_vendor (vendor_id),
  CONSTRAINT fk_purchase_requisition_vendor FOREIGN KEY (vendor_id)
    REFERENCES vendor (vendor_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Buying transport capacity and services: purchase requisition';

CREATE TABLE requisition_line (
  requisition_line_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  purchase_requisition_id    BIGINT UNSIGNED NOT NULL,
  required_by                DATE,
  currency_code              CHAR(3),
  unit_price                 DECIMAL(12,4),
  quantity_ordered           DECIMAL(12,3),
  quantity_received          DECIMAL(12,3),
  uom_code                   VARCHAR(8),
  supplier_reference         VARCHAR(48),
  buyer_code                 VARCHAR(24),
  approval_level             TINYINT UNSIGNED,
  approved_on                DATETIME,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (requisition_line_id),
  KEY idx_requisition_line_currency_code (currency_code),
  KEY fk_requisition_line_purchase_requisition (purchase_requisition_id),
  CONSTRAINT fk_requisition_line_purchase_requisition FOREIGN KEY (purchase_requisition_id)
    REFERENCES purchase_requisition (purchase_requisition_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Buying transport capacity and services: requisition line';

CREATE TABLE purchase_order (
  purchase_order_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vendor_id                  BIGINT UNSIGNED NOT NULL,
  quantity_ordered           DECIMAL(12,3),
  quantity_received          DECIMAL(12,3),
  uom_code                   VARCHAR(8),
  supplier_reference         VARCHAR(48),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (purchase_order_id),
  KEY idx_purchase_order_uom_code (uom_code),
  KEY fk_purchase_order_vendor (vendor_id),
  CONSTRAINT fk_purchase_order_vendor FOREIGN KEY (vendor_id)
    REFERENCES vendor (vendor_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Buying transport capacity and services: purchase order';

CREATE TABLE purchase_order_line (
  purchase_order_line_id     BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  purchase_order_id          BIGINT UNSIGNED NOT NULL,
  supplier_reference         VARCHAR(48),
  buyer_code                 VARCHAR(24),
  approval_level             TINYINT UNSIGNED,
  approved_on                DATETIME,
  delivery_terms             VARCHAR(32),
  payment_terms              VARCHAR(32),
  budget_code                VARCHAR(16),
  justification              TEXT,
  scoring                    JSON,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (purchase_order_line_id),
  KEY idx_purchase_order_line_supplier_reference (supplier_reference),
  KEY fk_purchase_order_line_purchase_order (purchase_order_id),
  CONSTRAINT fk_purchase_order_line_purchase_order FOREIGN KEY (purchase_order_id)
    REFERENCES purchase_order (purchase_order_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE goods_receipt (
  goods_receipt_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  purchase_order_id          BIGINT UNSIGNED NOT NULL,
  approved_on                DATETIME,
  delivery_terms             VARCHAR(32),
  payment_terms              VARCHAR(32),
  budget_code                VARCHAR(16),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (goods_receipt_id),
  KEY idx_goods_receipt_delivery_terms (delivery_terms),
  KEY fk_goods_receipt_purchase_order (purchase_order_id),
  CONSTRAINT fk_goods_receipt_purchase_order FOREIGN KEY (purchase_order_id)
    REFERENCES purchase_order (purchase_order_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE goods_receipt_line (
  goods_receipt_line_id      BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  goods_receipt_id           BIGINT UNSIGNED NOT NULL,
  currency_code              CHAR(3),
  unit_price                 DECIMAL(12,4),
  quantity_ordered           DECIMAL(12,3),
  quantity_received          DECIMAL(12,3),
  uom_code                   VARCHAR(8),
  supplier_reference         VARCHAR(48),
  buyer_code                 VARCHAR(24),
  approval_level             TINYINT UNSIGNED,
  unit_price_rounded         DECIMAL(18,2) AS (ROUND(unit_price, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (goods_receipt_line_id),
  KEY idx_goods_receipt_line_currency_code (currency_code),
  KEY idx_goods_receipt_line_currency_code_created (currency_code, created_at),
  KEY fk_goods_receipt_line_goods_receipt (goods_receipt_id),
  CONSTRAINT fk_goods_receipt_line_goods_receipt FOREIGN KEY (goods_receipt_id)
    REFERENCES goods_receipt (goods_receipt_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Buying transport capacity and services: goods receipt line';

CREATE TABLE vendor_contract (
  vendor_contract_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vendor_id                  BIGINT UNSIGNED NOT NULL,
  document_no                VARCHAR(32),
  status                     VARCHAR(24),
  raised_on                  DATE,
  required_by                DATE,
  currency_code              CHAR(3),
  unit_price                 DECIMAL(12,4),
  quantity_ordered           DECIMAL(12,3),
  quantity_received          DECIMAL(12,3),
  uom_code                   VARCHAR(8),
  supplier_reference         VARCHAR(48),
  buyer_code                 VARCHAR(24),
  approval_level             TINYINT UNSIGNED,
  approved_on                DATETIME,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (vendor_contract_id),
  UNIQUE KEY ux_vendor_contract_document_no (document_no),
  KEY fk_vendor_contract_vendor (vendor_id),
  CONSTRAINT fk_vendor_contract_vendor FOREIGN KEY (vendor_id)
    REFERENCES vendor (vendor_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Buying transport capacity and services: vendor contract';

CREATE TABLE vendor_contract_line (
  vendor_contract_line_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vendor_contract_id         BIGINT UNSIGNED NOT NULL,
  supplier_reference         VARCHAR(48),
  buyer_code                 VARCHAR(24),
  approval_level             TINYINT UNSIGNED,
  approved_on                DATETIME,
  delivery_terms             VARCHAR(32),
  payment_terms              VARCHAR(32),
  budget_code                VARCHAR(16),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (vendor_contract_line_id),
  KEY idx_vendor_contract_line_supplier_reference (supplier_reference),
  KEY fk_vendor_contract_line_vendor_contract (vendor_contract_id),
  CONSTRAINT fk_vendor_contract_line_vendor_contract FOREIGN KEY (vendor_contract_id)
    REFERENCES vendor_contract (vendor_contract_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Buying transport capacity and services: vendor contract line';

CREATE TABLE supplier_rate (
  supplier_rate_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vendor_id                  BIGINT UNSIGNED NOT NULL,
  required_by                DATE,
  currency_code              CHAR(3),
  unit_price                 DECIMAL(12,4),
  quantity_ordered           DECIMAL(12,3),
  quantity_received          DECIMAL(12,3),
  uom_code                   VARCHAR(8),
  supplier_reference         VARCHAR(48),
  buyer_code                 VARCHAR(24),
  approval_level             TINYINT UNSIGNED,
  approved_on                DATETIME,
  delivery_terms             VARCHAR(32),
  payment_terms              VARCHAR(32),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (supplier_rate_id),
  KEY idx_supplier_rate_currency_code (currency_code),
  KEY fk_supplier_rate_vendor (vendor_id),
  CONSTRAINT fk_supplier_rate_vendor FOREIGN KEY (vendor_id)
    REFERENCES vendor (vendor_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE tender (
  tender_id                  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vendor_id                  BIGINT UNSIGNED NOT NULL,
  status                     VARCHAR(24),
  raised_on                  DATE,
  required_by                DATE,
  currency_code              CHAR(3),
  unit_price                 DECIMAL(12,4),
  quantity_ordered           DECIMAL(12,3),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (tender_id),
  KEY idx_tender_status (status),
  KEY fk_tender_vendor (vendor_id),
  CONSTRAINT fk_tender_vendor FOREIGN KEY (vendor_id)
    REFERENCES vendor (vendor_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE tender_response (
  tender_response_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  tender_id                  BIGINT UNSIGNED NOT NULL,
  quantity_ordered           DECIMAL(12,3),
  quantity_received          DECIMAL(12,3),
  uom_code                   VARCHAR(8),
  supplier_reference         VARCHAR(48),
  buyer_code                 VARCHAR(24),
  approval_level             TINYINT UNSIGNED,
  approved_on                DATETIME,
  delivery_terms             VARCHAR(32),
  payment_terms              VARCHAR(32),
  budget_code                VARCHAR(16),
  justification              TEXT,
  quantity_ordered_rounded   DECIMAL(18,2) AS (ROUND(quantity_ordered, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (tender_response_id),
  KEY idx_tender_response_uom_code (uom_code),
  KEY fk_tender_response_tender (tender_id),
  CONSTRAINT fk_tender_response_tender FOREIGN KEY (tender_id)
    REFERENCES tender (tender_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Buying transport capacity and services: tender response';

CREATE TABLE spend_category (
  spend_category_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  charge_code_id             BIGINT UNSIGNED NOT NULL,
  unit_price                 DECIMAL(12,4),
  quantity_ordered           DECIMAL(12,3),
  quantity_received          DECIMAL(12,3),
  uom_code                   VARCHAR(8),
  supplier_reference         VARCHAR(48),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (spend_category_id),
  KEY idx_spend_category_uom_code (uom_code),
  KEY fk_spend_category_charge_code (charge_code_id),
  CONSTRAINT fk_spend_category_charge_code FOREIGN KEY (charge_code_id)
    REFERENCES charge_code (charge_code_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Buying transport capacity and services: spend category';

CREATE TABLE employee (
  employee_id                BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  personnel_no               VARCHAR(16),
  given_name                 VARCHAR(80),
  family_name                VARCHAR(80),
  date_of_birth              DATE,
  nationality                CHAR(2),
  job_title                  VARCHAR(80),
  department                 VARCHAR(64),
  started_on                 DATE,
  ended_on                   DATE,
  hours_worked               DECIMAL(7,2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (employee_id),
  UNIQUE KEY ux_employee_personnel_no (personnel_no),
  KEY fk_employee_organisation (organisation_id),
  CONSTRAINT fk_employee_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Staff, crew and drivers: employee';

CREATE TABLE employee_role (
  employee_role_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  employee_id                BIGINT UNSIGNED NOT NULL,
  date_of_birth              DATE,
  nationality                CHAR(2),
  job_title                  VARCHAR(80),
  department                 VARCHAR(64),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (employee_role_id),
  KEY idx_employee_role_nationality (nationality),
  KEY fk_employee_role_employee (employee_id),
  CONSTRAINT fk_employee_role_employee FOREIGN KEY (employee_id)
    REFERENCES employee (employee_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE employee_absence (
  employee_absence_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  employee_id                BIGINT UNSIGNED NOT NULL,
  department                 VARCHAR(64),
  started_on                 DATE,
  ended_on                   DATE,
  hours_worked               DECIMAL(7,2),
  hourly_rate                DECIMAL(9,2),
  certificate_no             VARCHAR(48),
  issued_on                  DATE,
  expires_on                 DATE,
  contact_email              VARCHAR(160),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (employee_absence_id),
  KEY idx_employee_absence_department (department),
  KEY fk_employee_absence_employee (employee_id),
  CONSTRAINT fk_employee_absence_employee FOREIGN KEY (employee_id)
    REFERENCES employee (employee_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE crew_member (
  crew_member_id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vessel_id                  BIGINT UNSIGNED NOT NULL,
  hours_worked               DECIMAL(7,2),
  hourly_rate                DECIMAL(9,2),
  certificate_no             VARCHAR(48),
  issued_on                  DATE,
  hours_worked_rounded       DECIMAL(18,2) AS (ROUND(hours_worked, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (crew_member_id),
  UNIQUE KEY ux_crew_member_certificate_no (certificate_no),
  KEY idx_crew_member_certificate_no_created (certificate_no, created_at),
  KEY fk_crew_member_vessel (vessel_id),
  CONSTRAINT fk_crew_member_vessel FOREIGN KEY (vessel_id)
    REFERENCES vessel (vessel_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Staff, crew and drivers: crew member';

CREATE TABLE crew_certificate (
  crew_certificate_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  crew_member_id             BIGINT UNSIGNED NOT NULL,
  given_name                 VARCHAR(80),
  family_name                VARCHAR(80),
  date_of_birth              DATE,
  nationality                CHAR(2),
  job_title                  VARCHAR(80),
  department                 VARCHAR(64),
  started_on                 DATE,
  ended_on                   DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (crew_certificate_id),
  KEY idx_crew_certificate_given_name (given_name),
  KEY fk_crew_certificate_crew_member (crew_member_id),
  CONSTRAINT fk_crew_certificate_crew_member FOREIGN KEY (crew_member_id)
    REFERENCES crew_member (crew_member_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Staff, crew and drivers: crew certificate';

CREATE TABLE crew_rotation (
  crew_rotation_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  crew_member_id             BIGINT UNSIGNED NOT NULL,
  date_of_birth              DATE,
  nationality                CHAR(2),
  job_title                  VARCHAR(80),
  department                 VARCHAR(64),
  started_on                 DATE,
  ended_on                   DATE,
  hours_worked               DECIMAL(7,2),
  hourly_rate                DECIMAL(9,2),
  certificate_no             VARCHAR(48),
  issued_on                  DATE,
  expires_on                 DATE,
  contact_email              VARCHAR(160),
  emergency_phone            VARCHAR(32),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (crew_rotation_id),
  KEY idx_crew_rotation_nationality (nationality),
  KEY fk_crew_rotation_crew_member (crew_member_id),
  CONSTRAINT fk_crew_rotation_crew_member FOREIGN KEY (crew_member_id)
    REFERENCES crew_member (crew_member_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Staff, crew and drivers: crew rotation';

CREATE TABLE driver (
  driver_id                  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  department                 VARCHAR(64),
  started_on                 DATE,
  ended_on                   DATE,
  hours_worked               DECIMAL(7,2),
  hourly_rate                DECIMAL(9,2),
  certificate_no             VARCHAR(48),
  issued_on                  DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (driver_id),
  KEY idx_driver_department (department),
  KEY fk_driver_organisation (organisation_id),
  CONSTRAINT fk_driver_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE driver_licence (
  driver_licence_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  driver_id                  BIGINT UNSIGNED NOT NULL,
  personnel_no               VARCHAR(16),
  given_name                 VARCHAR(80),
  family_name                VARCHAR(80),
  date_of_birth              DATE,
  nationality                CHAR(2),
  job_title                  VARCHAR(80),
  department                 VARCHAR(64),
  started_on                 DATE,
  ended_on                   DATE,
  hours_worked               DECIMAL(7,2),
  hourly_rate                DECIMAL(9,2),
  certificate_no             VARCHAR(48),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (driver_licence_id),
  UNIQUE KEY ux_driver_licence_personnel_no (personnel_no),
  KEY fk_driver_licence_driver (driver_id),
  CONSTRAINT fk_driver_licence_driver FOREIGN KEY (driver_id)
    REFERENCES driver (driver_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE driver_hours (
  driver_hours_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  driver_id                  BIGINT UNSIGNED NOT NULL,
  certificate_no             VARCHAR(48),
  issued_on                  DATE,
  expires_on                 DATE,
  contact_email              VARCHAR(160),
  emergency_phone            VARCHAR(32),
  comments                   TEXT,
  issued_on_year             SMALLINT UNSIGNED AS (YEAR(issued_on)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (driver_hours_id),
  UNIQUE KEY ux_driver_hours_certificate_no (certificate_no),
  KEY fk_driver_hours_driver (driver_id),
  CONSTRAINT fk_driver_hours_driver FOREIGN KEY (driver_id)
    REFERENCES driver (driver_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Staff, crew and drivers: driver hours';

CREATE TABLE training_record (
  training_record_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  employee_id                BIGINT UNSIGNED NOT NULL,
  date_of_birth              DATE,
  nationality                CHAR(2),
  job_title                  VARCHAR(80),
  department                 VARCHAR(64),
  started_on                 DATE,
  ended_on                   DATE,
  hours_worked               DECIMAL(7,2),
  hourly_rate                DECIMAL(9,2),
  certificate_no             VARCHAR(48),
  issued_on                  DATE,
  expires_on                 DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (training_record_id),
  KEY idx_training_record_nationality (nationality),
  KEY fk_training_record_employee (employee_id),
  CONSTRAINT fk_training_record_employee FOREIGN KEY (employee_id)
    REFERENCES employee (employee_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Staff, crew and drivers: training record';

CREATE TABLE timesheet (
  timesheet_id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  employee_id                BIGINT UNSIGNED NOT NULL,
  family_name                VARCHAR(80),
  date_of_birth              DATE,
  nationality                CHAR(2),
  job_title                  VARCHAR(80),
  department                 VARCHAR(64),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (timesheet_id),
  KEY idx_timesheet_family_name (family_name),
  KEY fk_timesheet_employee (employee_id),
  CONSTRAINT fk_timesheet_employee FOREIGN KEY (employee_id)
    REFERENCES employee (employee_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Staff, crew and drivers: timesheet';

CREATE TABLE payroll_line (
  payroll_line_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  employee_id                BIGINT UNSIGNED NOT NULL,
  department                 VARCHAR(64),
  started_on                 DATE,
  ended_on                   DATE,
  hours_worked               DECIMAL(7,2),
  hourly_rate                DECIMAL(9,2),
  certificate_no             VARCHAR(48),
  issued_on                  DATE,
  expires_on                 DATE,
  contact_email              VARCHAR(160),
  emergency_phone            VARCHAR(32),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (payroll_line_id),
  KEY idx_payroll_line_department (department),
  KEY fk_payroll_line_employee (employee_id),
  CONSTRAINT fk_payroll_line_employee FOREIGN KEY (employee_id)
    REFERENCES employee (employee_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE document (
  document_id                BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  document_type_id           BIGINT UNSIGNED NOT NULL,
  document_no                VARCHAR(48),
  title                      VARCHAR(160),
  status                     VARCHAR(24),
  issued_on                  DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (document_id),
  UNIQUE KEY ux_document_document_no (document_no),
  KEY fk_document_document_type (document_type_id),
  CONSTRAINT fk_document_document_type FOREIGN KEY (document_type_id)
    REFERENCES document_type (document_type_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE document_version (
  document_version_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  document_id                BIGINT UNSIGNED NOT NULL,
  issued_on                  DATE,
  version_no                 SMALLINT UNSIGNED,
  mime_type                  VARCHAR(80),
  byte_size                  BIGINT UNSIGNED,
  checksum_sha256            CHAR(64),
  storage_key                VARCHAR(160),
  segment_tag                VARCHAR(8),
  segment_position           INT UNSIGNED,
  interchange_ref            VARCHAR(32),
  version_no_rounded         DECIMAL(18,2) AS (ROUND(version_no, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (document_version_id),
  KEY idx_document_version_mime_type (mime_type),
  KEY idx_document_version_mime_type_created (mime_type, created_at),
  KEY fk_document_version_document (document_id),
  CONSTRAINT fk_document_version_document FOREIGN KEY (document_id)
    REFERENCES document (document_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Transport documents and EDI traffic: document version';

CREATE TABLE document_link (
  document_link_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  document_id                BIGINT UNSIGNED NOT NULL,
  byte_size                  BIGINT UNSIGNED,
  checksum_sha256            CHAR(64),
  storage_key                VARCHAR(160),
  segment_tag                VARCHAR(8),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (document_link_id),
  KEY idx_document_link_checksum_sha256 (checksum_sha256),
  KEY fk_document_link_document (document_id),
  CONSTRAINT fk_document_link_document FOREIGN KEY (document_id)
    REFERENCES document (document_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Transport documents and EDI traffic: document link';

CREATE TABLE bill_of_lading (
  bill_of_lading_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  segment_tag                VARCHAR(8),
  segment_position           INT UNSIGNED,
  interchange_ref            VARCHAR(32),
  sender_id                  VARCHAR(48),
  receiver_id                VARCHAR(48),
  processed_at               DATETIME,
  error_code                 VARCHAR(16),
  payload                    TEXT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (bill_of_lading_id),
  KEY idx_bill_of_lading_segment_tag (segment_tag),
  KEY fk_bill_of_lading_consignment (consignment_id),
  CONSTRAINT fk_bill_of_lading_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Transport documents and EDI traffic: bill of lading';

CREATE TABLE bill_of_lading_clause (
  bill_of_lading_clause_id   BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  bill_of_lading_id          BIGINT UNSIGNED NOT NULL,
  document_no                VARCHAR(48),
  title                      VARCHAR(160),
  status                     VARCHAR(24),
  issued_on                  DATE,
  version_no                 SMALLINT UNSIGNED,
  mime_type                  VARCHAR(80),
  byte_size                  BIGINT UNSIGNED,
  checksum_sha256            CHAR(64),
  storage_key                VARCHAR(160),
  segment_tag                VARCHAR(8),
  segment_position           INT UNSIGNED,
  interchange_ref            VARCHAR(32),
  sender_id                  VARCHAR(48),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (bill_of_lading_clause_id),
  UNIQUE KEY ux_bill_of_lading_clause_document_no (document_no),
  KEY fk_bill_of_lading_clause_bill_of_lading (bill_of_lading_id),
  CONSTRAINT fk_bill_of_lading_clause_bill_of_lading FOREIGN KEY (bill_of_lading_id)
    REFERENCES bill_of_lading (bill_of_lading_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE air_waybill (
  air_waybill_id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  issued_on                  DATE,
  version_no                 SMALLINT UNSIGNED,
  mime_type                  VARCHAR(80),
  byte_size                  BIGINT UNSIGNED,
  checksum_sha256            CHAR(64),
  storage_key                VARCHAR(160),
  segment_tag                VARCHAR(8),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (air_waybill_id),
  KEY idx_air_waybill_mime_type (mime_type),
  KEY fk_air_waybill_consignment (consignment_id),
  CONSTRAINT fk_air_waybill_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE packing_list (
  packing_list_id            BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  version_no                 SMALLINT UNSIGNED,
  mime_type                  VARCHAR(80),
  byte_size                  BIGINT UNSIGNED,
  checksum_sha256            CHAR(64),
  storage_key                VARCHAR(160),
  segment_tag                VARCHAR(8),
  segment_position           INT UNSIGNED,
  interchange_ref            VARCHAR(32),
  sender_id                  VARCHAR(48),
  receiver_id                VARCHAR(48),
  processed_at               DATETIME,
  error_code                 VARCHAR(16),
  version_no_rounded         DECIMAL(18,2) AS (ROUND(version_no, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (packing_list_id),
  KEY idx_packing_list_mime_type (mime_type),
  KEY fk_packing_list_consignment (consignment_id),
  CONSTRAINT fk_packing_list_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Transport documents and EDI traffic: packing list';

CREATE TABLE commercial_invoice_doc (
  commercial_invoice_doc_id  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  storage_key                VARCHAR(160),
  segment_tag                VARCHAR(8),
  segment_position           INT UNSIGNED,
  interchange_ref            VARCHAR(32),
  sender_id                  VARCHAR(48),
  receiver_id                VARCHAR(48),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (commercial_invoice_doc_id),
  KEY idx_commercial_invoice_doc_storage_key (storage_key),
  KEY fk_commercial_invoice_doc_consignment (consignment_id),
  CONSTRAINT fk_commercial_invoice_doc_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Transport documents and EDI traffic: commercial invoice doc';

CREATE TABLE certificate_of_origin (
  certificate_of_origin_id   BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  consignment_id             BIGINT UNSIGNED NOT NULL,
  document_no                VARCHAR(48),
  title                      VARCHAR(160),
  status                     VARCHAR(24),
  issued_on                  DATE,
  version_no                 SMALLINT UNSIGNED,
  mime_type                  VARCHAR(80),
  byte_size                  BIGINT UNSIGNED,
  checksum_sha256            CHAR(64),
  storage_key                VARCHAR(160),
  segment_tag                VARCHAR(8),
  segment_position           INT UNSIGNED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (certificate_of_origin_id),
  UNIQUE KEY ux_certificate_of_origin_document_no (document_no),
  KEY fk_certificate_of_origin_consignment (consignment_id),
  CONSTRAINT fk_certificate_of_origin_consignment FOREIGN KEY (consignment_id)
    REFERENCES consignment (consignment_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Transport documents and EDI traffic: certificate of origin';

CREATE TABLE edi_message (
  edi_message_id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  edi_standard_id            BIGINT UNSIGNED NOT NULL,
  receiver_id                VARCHAR(48),
  processed_at               DATETIME,
  error_code                 VARCHAR(16),
  payload                    TEXT,
  routing                    JSON,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (edi_message_id),
  KEY idx_edi_message_receiver_id (receiver_id),
  KEY fk_edi_message_edi_standard (edi_standard_id),
  CONSTRAINT fk_edi_message_edi_standard FOREIGN KEY (edi_standard_id)
    REFERENCES edi_standard (edi_standard_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE edi_segment (
  edi_segment_id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  edi_message_id             BIGINT UNSIGNED NOT NULL,
  issued_on                  DATE,
  version_no                 SMALLINT UNSIGNED,
  mime_type                  VARCHAR(80),
  byte_size                  BIGINT UNSIGNED,
  checksum_sha256            CHAR(64),
  storage_key                VARCHAR(160),
  segment_tag                VARCHAR(8),
  segment_position           INT UNSIGNED,
  interchange_ref            VARCHAR(32),
  sender_id                  VARCHAR(48),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (edi_segment_id),
  KEY idx_edi_segment_mime_type (mime_type),
  KEY fk_edi_segment_edi_message (edi_message_id),
  CONSTRAINT fk_edi_segment_edi_message FOREIGN KEY (edi_message_id)
    REFERENCES edi_message (edi_message_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE edi_partner_profile (
  edi_partner_profile_id     BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  issued_on                  DATE,
  version_no                 SMALLINT UNSIGNED,
  mime_type                  VARCHAR(80),
  byte_size                  BIGINT UNSIGNED,
  version_no_rounded         DECIMAL(18,2) AS (ROUND(version_no, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (edi_partner_profile_id),
  KEY idx_edi_partner_profile_mime_type (mime_type),
  KEY idx_edi_partner_profile_mime_type_created (mime_type, created_at),
  KEY fk_edi_partner_profile_organisation (organisation_id),
  CONSTRAINT fk_edi_partner_profile_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Transport documents and EDI traffic: edi partner profile';

CREATE TABLE edi_error (
  edi_error_id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  edi_message_id             BIGINT UNSIGNED NOT NULL,
  byte_size                  BIGINT UNSIGNED,
  checksum_sha256            CHAR(64),
  storage_key                VARCHAR(160),
  segment_tag                VARCHAR(8),
  segment_position           INT UNSIGNED,
  interchange_ref            VARCHAR(32),
  sender_id                  VARCHAR(48),
  receiver_id                VARCHAR(48),
  processed_at               DATETIME,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (edi_error_id),
  KEY idx_edi_error_checksum_sha256 (checksum_sha256),
  KEY fk_edi_error_edi_message (edi_message_id),
  CONSTRAINT fk_edi_error_edi_message FOREIGN KEY (edi_message_id)
    REFERENCES edi_message (edi_message_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Transport documents and EDI traffic: edi error';

CREATE TABLE claim (
  claim_id                   BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  claim_category_id          BIGINT UNSIGNED NOT NULL,
  case_reference             VARCHAR(32),
  status                     VARCHAR(24),
  opened_on                  DATE,
  closed_on                  DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (claim_id),
  UNIQUE KEY ux_claim_case_reference (case_reference),
  KEY fk_claim_claim_category (claim_category_id),
  CONSTRAINT fk_claim_claim_category FOREIGN KEY (claim_category_id)
    REFERENCES claim_category (claim_category_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Claims, incidents and service performance: claim';

CREATE TABLE claim_line (
  claim_line_id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  claim_id                   BIGINT UNSIGNED NOT NULL,
  closed_on                  DATE,
  claimed_amount             DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  currency_code              CHAR(3),
  liability_pct              DECIMAL(5,2),
  severity                   VARCHAR(16),
  root_cause                 VARCHAR(120),
  measured_on                DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (claim_line_id),
  KEY idx_claim_line_currency_code (currency_code),
  KEY fk_claim_line_claim (claim_id),
  CONSTRAINT fk_claim_line_claim FOREIGN KEY (claim_id)
    REFERENCES claim (claim_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE claim_document (
  claim_document_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  claim_id                   BIGINT UNSIGNED NOT NULL,
  case_reference             VARCHAR(32),
  status                     VARCHAR(24),
  opened_on                  DATE,
  closed_on                  DATE,
  claimed_amount             DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  currency_code              CHAR(3),
  liability_pct              DECIMAL(5,2),
  severity                   VARCHAR(16),
  root_cause                 VARCHAR(120),
  measured_on                DATE,
  measured_value             DECIMAL(14,4),
  target_value               DECIMAL(14,4),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (claim_document_id),
  UNIQUE KEY ux_claim_document_case_reference (case_reference),
  KEY fk_claim_document_claim (claim_id),
  CONSTRAINT fk_claim_document_claim FOREIGN KEY (claim_id)
    REFERENCES claim (claim_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE survey_report (
  survey_report_id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  claim_id                   BIGINT UNSIGNED NOT NULL,
  root_cause                 VARCHAR(120),
  measured_on                DATE,
  measured_value             DECIMAL(14,4),
  target_value               DECIMAL(14,4),
  unit_label                 VARCHAR(24),
  responsible_team           VARCHAR(64),
  due_on                     DATE,
  measured_value_rounded     DECIMAL(18,2) AS (ROUND(measured_value, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (survey_report_id),
  KEY idx_survey_report_root_cause (root_cause),
  KEY fk_survey_report_claim (claim_id),
  CONSTRAINT fk_survey_report_claim FOREIGN KEY (claim_id)
    REFERENCES claim (claim_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Claims, incidents and service performance: survey report';

CREATE TABLE insurance_policy (
  insurance_policy_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  organisation_id            BIGINT UNSIGNED NOT NULL,
  settled_amount             DECIMAL(14,2),
  currency_code              CHAR(3),
  liability_pct              DECIMAL(5,2),
  severity                   VARCHAR(16),
  root_cause                 VARCHAR(120),
  measured_on                DATE,
  measured_value             DECIMAL(14,4),
  target_value               DECIMAL(14,4),
  unit_label                 VARCHAR(24),
  responsible_team           VARCHAR(64),
  due_on                     DATE,
  findings                   TEXT,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (insurance_policy_id),
  KEY idx_insurance_policy_currency_code (currency_code),
  KEY fk_insurance_policy_organisation (organisation_id),
  CONSTRAINT fk_insurance_policy_organisation FOREIGN KEY (organisation_id)
    REFERENCES organisation (organisation_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Claims, incidents and service performance: insurance policy';

CREATE TABLE insurance_cover (
  insurance_cover_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  insurance_policy_id        BIGINT UNSIGNED NOT NULL,
  opened_on                  DATE,
  closed_on                  DATE,
  claimed_amount             DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  currency_code              CHAR(3),
  liability_pct              DECIMAL(5,2),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (insurance_cover_id),
  KEY idx_insurance_cover_currency_code (currency_code),
  KEY fk_insurance_cover_insurance_policy (insurance_policy_id),
  CONSTRAINT fk_insurance_cover_insurance_policy FOREIGN KEY (insurance_policy_id)
    REFERENCES insurance_policy (insurance_policy_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Claims, incidents and service performance: insurance cover';

CREATE TABLE incident (
  incident_id                BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  transport_order_id         BIGINT UNSIGNED NOT NULL,
  opened_on                  DATE,
  closed_on                  DATE,
  claimed_amount             DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  currency_code              CHAR(3),
  liability_pct              DECIMAL(5,2),
  severity                   VARCHAR(16),
  root_cause                 VARCHAR(120),
  measured_on                DATE,
  measured_value             DECIMAL(14,4),
  target_value               DECIMAL(14,4),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (incident_id),
  KEY idx_incident_currency_code (currency_code),
  KEY fk_incident_transport_order (transport_order_id),
  CONSTRAINT fk_incident_transport_order FOREIGN KEY (transport_order_id)
    REFERENCES transport_order (transport_order_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE incident_action (
  incident_action_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  incident_id                BIGINT UNSIGNED NOT NULL,
  liability_pct              DECIMAL(5,2),
  severity                   VARCHAR(16),
  root_cause                 VARCHAR(120),
  measured_on                DATE,
  measured_value             DECIMAL(14,4),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (incident_action_id),
  KEY idx_incident_action_severity (severity),
  KEY fk_incident_action_incident (incident_id),
  CONSTRAINT fk_incident_action_incident FOREIGN KEY (incident_id)
    REFERENCES incident (incident_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE nonconformance (
  nonconformance_id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  incident_id                BIGINT UNSIGNED NOT NULL,
  currency_code              CHAR(3),
  liability_pct              DECIMAL(5,2),
  severity                   VARCHAR(16),
  root_cause                 VARCHAR(120),
  measured_on                DATE,
  measured_value             DECIMAL(14,4),
  target_value               DECIMAL(14,4),
  unit_label                 VARCHAR(24),
  responsible_team           VARCHAR(64),
  due_on                     DATE,
  liability_pct_rounded      DECIMAL(18,2) AS (ROUND(liability_pct, 2)) STORED,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (nonconformance_id),
  KEY idx_nonconformance_currency_code (currency_code),
  KEY idx_nonconformance_currency_code_created (currency_code, created_at),
  KEY fk_nonconformance_incident (incident_id),
  CONSTRAINT fk_nonconformance_incident FOREIGN KEY (incident_id)
    REFERENCES incident (incident_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Claims, incidents and service performance: nonconformance';

CREATE TABLE corrective_action (
  corrective_action_id       BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  nonconformance_id          BIGINT UNSIGNED NOT NULL,
  target_value               DECIMAL(14,4),
  unit_label                 VARCHAR(24),
  responsible_team           VARCHAR(64),
  due_on                     DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (corrective_action_id),
  KEY idx_corrective_action_unit_label (unit_label),
  KEY fk_corrective_action_nonconformance (nonconformance_id),
  CONSTRAINT fk_corrective_action_nonconformance FOREIGN KEY (nonconformance_id)
    REFERENCES nonconformance (nonconformance_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Claims, incidents and service performance: corrective action';

CREATE TABLE service_kpi (
  service_kpi_id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  service_level_id           BIGINT UNSIGNED NOT NULL,
  case_reference             VARCHAR(32),
  status                     VARCHAR(24),
  opened_on                  DATE,
  closed_on                  DATE,
  claimed_amount             DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  currency_code              CHAR(3),
  liability_pct              DECIMAL(5,2),
  severity                   VARCHAR(16),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (service_kpi_id),
  UNIQUE KEY ux_service_kpi_case_reference (case_reference),
  KEY fk_service_kpi_service_level (service_level_id),
  CONSTRAINT fk_service_kpi_service_level FOREIGN KEY (service_level_id)
    REFERENCES service_level (service_level_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Claims, incidents and service performance: service kpi';

CREATE TABLE kpi_measurement (
  kpi_measurement_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  service_kpi_id             BIGINT UNSIGNED NOT NULL,
  closed_on                  DATE,
  claimed_amount             DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  currency_code              CHAR(3),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (kpi_measurement_id),
  KEY idx_kpi_measurement_currency_code (currency_code),
  KEY fk_kpi_measurement_service_kpi (service_kpi_id),
  CONSTRAINT fk_kpi_measurement_service_kpi FOREIGN KEY (service_kpi_id)
    REFERENCES service_kpi (service_kpi_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE transit_time_actual (
  transit_time_actual_id     BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  trade_lane_id              BIGINT UNSIGNED NOT NULL,
  closed_on                  DATE,
  claimed_amount             DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  currency_code              CHAR(3),
  liability_pct              DECIMAL(5,2),
  severity                   VARCHAR(16),
  root_cause                 VARCHAR(120),
  measured_on                DATE,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (transit_time_actual_id),
  KEY idx_transit_time_actual_currency_code (currency_code),
  KEY fk_transit_time_actual_trade_lane (trade_lane_id),
  CONSTRAINT fk_transit_time_actual_trade_lane FOREIGN KEY (trade_lane_id)
    REFERENCES trade_lane (trade_lane_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE lane_performance (
  lane_performance_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  trade_lane_id              BIGINT UNSIGNED NOT NULL,
  closed_on                  DATE,
  claimed_amount             DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  currency_code              CHAR(3),
  liability_pct              DECIMAL(5,2),
  severity                   VARCHAR(16),
  root_cause                 VARCHAR(120),
  measured_on                DATE,
  measured_value             DECIMAL(14,4),
  target_value               DECIMAL(14,4),
  unit_label                 VARCHAR(24),
  responsible_team           VARCHAR(64),
  due_on                     DATE,
  claimed_amount_rounded     DECIMAL(18,2) AS (ROUND(claimed_amount, 2)) VIRTUAL,
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (lane_performance_id),
  KEY idx_lane_performance_currency_code (currency_code),
  KEY fk_lane_performance_trade_lane (trade_lane_id),
  CONSTRAINT fk_lane_performance_trade_lane FOREIGN KEY (trade_lane_id)
    REFERENCES trade_lane (trade_lane_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Claims, incidents and service performance: lane performance';

CREATE TABLE utilisation_snapshot (
  utilisation_snapshot_id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  vessel_id                  BIGINT UNSIGNED NOT NULL,
  currency_code              CHAR(3),
  liability_pct              DECIMAL(5,2),
  severity                   VARCHAR(16),
  root_cause                 VARCHAR(120),
  measured_on                DATE,
  measured_value             DECIMAL(14,4),
  target_value               DECIMAL(14,4),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (utilisation_snapshot_id),
  KEY idx_utilisation_snapshot_currency_code (currency_code),
  KEY fk_utilisation_snapshot_vessel (vessel_id),
  CONSTRAINT fk_utilisation_snapshot_vessel FOREIGN KEY (vessel_id)
    REFERENCES vessel (vessel_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Claims, incidents and service performance: utilisation snapshot';

CREATE TABLE emission_record (
  emission_record_id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  voyage_id                  BIGINT UNSIGNED NOT NULL,
  closed_on                  DATE,
  claimed_amount             DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  currency_code              CHAR(3),
  liability_pct              DECIMAL(5,2),
  severity                   VARCHAR(16),
  root_cause                 VARCHAR(120),
  measured_on                DATE,
  measured_value             DECIMAL(14,4),
  target_value               DECIMAL(14,4),
  unit_label                 VARCHAR(24),
  responsible_team           VARCHAR(64),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (emission_record_id),
  KEY idx_emission_record_currency_code (currency_code),
  KEY fk_emission_record_voyage (voyage_id),
  CONSTRAINT fk_emission_record_voyage FOREIGN KEY (voyage_id)
    REFERENCES voyage (voyage_id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Claims, incidents and service performance: emission record';

CREATE TABLE fuel_consumption (
  fuel_consumption_id        BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  voyage_id                  BIGINT UNSIGNED NOT NULL,
  root_cause                 VARCHAR(120),
  measured_on                DATE,
  measured_value             DECIMAL(14,4),
  target_value               DECIMAL(14,4),
  unit_label                 VARCHAR(24),
  responsible_team           VARCHAR(64),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (fuel_consumption_id),
  KEY idx_fuel_consumption_root_cause (root_cause),
  KEY fk_fuel_consumption_voyage (voyage_id),
  CONSTRAINT fk_fuel_consumption_voyage FOREIGN KEY (voyage_id)
    REFERENCES voyage (voyage_id) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE audit_log (
  audit_log_id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  audit_action_id            BIGINT UNSIGNED NOT NULL,
  closed_on                  DATE,
  claimed_amount             DECIMAL(14,2),
  settled_amount             DECIMAL(14,2),
  currency_code              CHAR(3),
  liability_pct              DECIMAL(5,2),
  severity                   VARCHAR(16),
  root_cause                 VARCHAR(120),
  measured_on                DATE,
  measured_value             DECIMAL(14,4),
  target_value               DECIMAL(14,4),
  unit_label                 VARCHAR(24),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (audit_log_id),
  KEY idx_audit_log_currency_code (currency_code),
  KEY fk_audit_log_audit_action (audit_action_id),
  CONSTRAINT fk_audit_log_audit_action FOREIGN KEY (audit_action_id)
    REFERENCES audit_action (audit_action_id) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- -----------------------------------------------------------------------------
-- Views
-- -----------------------------------------------------------------------------

CREATE VIEW v_port_summary AS
SELECT c.port_id,
       c.un_locode,
       c.created_at,
       p.code AS country_code
FROM port c
JOIN country p ON p.country_id = c.country_id;

CREATE VIEW v_locode_alias_summary AS
SELECT c.locode_alias_id,
       c.longitude,
       c.created_at,
       p.un_locode AS port_un_locode
FROM locode_alias c
JOIN port p ON p.port_id = c.port_id;

CREATE VIEW v_warehouse_site_summary AS
SELECT c.warehouse_site_id,
       c.contact_phone,
       c.created_at,
       p.code AS country_code
FROM warehouse_site c
JOIN country p ON p.country_id = c.country_id;

CREATE VIEW v_customer_credit_limit_summary AS
SELECT c.customer_credit_limit_id,
       c.trading_name,
       c.created_at,
       p.phone AS customer_phone
FROM customer_credit_limit c
JOIN customer p ON p.customer_id = c.customer_id;

CREATE VIEW v_notify_party_summary AS
SELECT c.notify_party_id,
       c.registration_no,
       c.created_at,
       p.legal_name AS organisation_legal_name
FROM notify_party c
JOIN organisation p ON p.organisation_id = c.organisation_id;

CREATE VIEW v_vessel_summary AS
SELECT c.vessel_id,
       c.imo_number,
       c.created_at,
       p.iso_alpha3 AS vessel_class_iso_alpha3
FROM vessel c
JOIN vessel_class p ON p.vessel_class_id = c.vessel_class_id;

CREATE VIEW v_port_call_summary AS
SELECT c.port_call_id,
       c.name,
       c.created_at,
       p.name AS voyage_leg_name
FROM port_call c
JOIN voyage_leg p ON p.voyage_leg_id = c.voyage_leg_id;

CREATE VIEW v_chassis_summary AS
SELECT c.chassis_id,
       c.flag_state,
       c.created_at,
       p.code AS equipment_grade_code
FROM chassis c
JOIN equipment_grade p ON p.equipment_grade_id = c.equipment_grade_id;

CREATE VIEW v_rate_card_summary AS
SELECT c.rate_card_id,
       c.incoterm_code,
       c.created_at,
       p.name AS trade_lane_name
FROM rate_card c
JOIN trade_lane p ON p.trade_lane_id = c.trade_lane_id;

CREATE VIEW v_booking_line_summary AS
SELECT c.booking_line_id,
       c.reference,
       c.created_at,
       p.incoterm_code AS booking_incoterm_code
FROM booking_line c
JOIN booking p ON p.booking_id = c.booking_id;

CREATE VIEW v_consignment_hazard_summary AS
SELECT c.consignment_hazard_id,
       c.incoterm_code,
       c.created_at,
       p.currency_code AS consignment_currency_code
FROM consignment_hazard c
JOIN consignment p ON p.consignment_id = c.consignment_id;

CREATE VIEW v_drayage_job_summary AS
SELECT c.drayage_job_id,
       c.vehicle_registration,
       c.created_at,
       p.job_reference AS transport_order_job_reference
FROM drayage_job c
JOIN transport_order p ON p.transport_order_id = c.transport_order_id;

CREATE VIEW v_loading_list_summary AS
SELECT c.loading_list_id,
       c.event_code,
       c.created_at,
       p.name AS port_call_name
FROM loading_list c
JOIN port_call p ON p.port_call_id = c.port_call_id;

CREATE VIEW v_warehouse_receipt_summary AS
SELECT c.warehouse_receipt_id,
       c.document_no,
       c.created_at,
       p.contact_phone AS warehouse_site_contact_phone
FROM warehouse_receipt c
JOIN warehouse_site p ON p.warehouse_site_id = c.warehouse_site_id;

CREATE VIEW v_count_line_summary AS
SELECT c.count_line_id,
       c.serial_no,
       c.created_at,
       p.document_no AS stock_count_document_no
FROM count_line c
JOIN stock_count p ON p.stock_count_id = c.stock_count_id;

CREATE VIEW v_storage_charge_summary AS
SELECT c.storage_charge_id,
       c.location_code,
       c.created_at,
       p.location_code AS stock_item_location_code
FROM storage_charge c
JOIN stock_item p ON p.stock_item_id = c.stock_item_id;

CREATE VIEW v_duty_calculation_summary AS
SELECT c.duty_calculation_id,
       c.guarantee_reference,
       c.created_at,
       p.commodity_code AS declaration_line_commodity_code
FROM duty_calculation c
JOIN declaration_line p ON p.declaration_line_id = c.declaration_line_id;

CREATE VIEW v_screening_hit_summary AS
SELECT c.screening_hit_id,
       c.country_of_origin,
       c.created_at,
       p.regime_code AS sanctions_screening_regime_code
FROM screening_hit c
JOIN sanctions_screening p ON p.sanctions_screening_id = c.sanctions_screening_id;

CREATE VIEW v_charge_allocation_summary AS
SELECT c.charge_allocation_id,
       c.currency_code,
       c.created_at,
       p.document_no AS charge_document_no
FROM charge_allocation c
JOIN charge p ON p.charge_id = c.charge_id;

CREATE VIEW v_payment_allocation_summary AS
SELECT c.payment_allocation_id,
       c.cost_centre,
       c.created_at,
       p.currency_code AS payment_currency_code
FROM payment_allocation c
JOIN payment p ON p.payment_id = c.payment_id;

CREATE VIEW v_purchase_invoice_line_summary AS
SELECT c.purchase_invoice_line_id,
       c.currency_code,
       c.created_at,
       p.currency_code AS purchase_invoice_currency_code
FROM purchase_invoice_line c
JOIN purchase_invoice p ON p.purchase_invoice_id = c.purchase_invoice_id;

CREATE VIEW v_requisition_line_summary AS
SELECT c.requisition_line_id,
       c.currency_code,
       c.created_at,
       p.document_no AS purchase_requisition_document_no
FROM requisition_line c
JOIN purchase_requisition p ON p.purchase_requisition_id = c.purchase_requisition_id;

CREATE VIEW v_vendor_contract_line_summary AS
SELECT c.vendor_contract_line_id,
       c.supplier_reference,
       c.created_at,
       p.document_no AS vendor_contract_document_no
FROM vendor_contract_line c
JOIN vendor_contract p ON p.vendor_contract_id = c.vendor_contract_id;

CREATE VIEW v_employee_role_summary AS
SELECT c.employee_role_id,
       c.nationality,
       c.created_at,
       p.personnel_no AS employee_personnel_no
FROM employee_role c
JOIN employee p ON p.employee_id = c.employee_id;

CREATE VIEW v_driver_licence_summary AS
SELECT c.driver_licence_id,
       c.personnel_no,
       c.created_at,
       p.department AS driver_department
FROM driver_licence c
JOIN driver p ON p.driver_id = c.driver_id;

CREATE VIEW v_document_version_summary AS
SELECT c.document_version_id,
       c.mime_type,
       c.created_at,
       p.document_no AS document_document_no
FROM document_version c
JOIN document p ON p.document_id = c.document_id;

CREATE VIEW v_commercial_invoice_doc_summary AS
SELECT c.commercial_invoice_doc_id,
       c.storage_key,
       c.created_at,
       p.currency_code AS consignment_currency_code
FROM commercial_invoice_doc c
JOIN consignment p ON p.consignment_id = c.consignment_id;

CREATE VIEW v_claim_summary AS
SELECT c.claim_id,
       c.case_reference,
       c.created_at,
       p.short_name AS claim_category_short_name
FROM claim c
JOIN claim_category p ON p.claim_category_id = c.claim_category_id;

CREATE VIEW v_incident_summary AS
SELECT c.incident_id,
       c.currency_code,
       c.created_at,
       p.job_reference AS transport_order_job_reference
FROM incident c
JOIN transport_order p ON p.transport_order_id = c.transport_order_id;

CREATE VIEW v_transit_time_actual_summary AS
SELECT c.transit_time_actual_id,
       c.currency_code,
       c.created_at,
       p.name AS trade_lane_name
FROM transit_time_actual c
JOIN trade_lane p ON p.trade_lane_id = c.trade_lane_id;

-- -----------------------------------------------------------------------------
-- Triggers
-- -----------------------------------------------------------------------------

CREATE TRIGGER trg_country_before_insert
BEFORE INSERT ON country
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_vessel_class_before_insert
BEFORE INSERT ON vessel_class
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_spoken_language_before_insert
BEFORE INSERT ON spoken_language
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_port_facility_before_insert
BEFORE INSERT ON port_facility
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_warehouse_site_before_insert
BEFORE INSERT ON warehouse_site
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_vendor_before_insert
BEFORE INSERT ON vendor
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_bank_account_before_insert
BEFORE INSERT ON bank_account
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_port_call_before_insert
BEFORE INSERT ON port_call
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_reefer_setpoint_before_insert
BEFORE INSERT ON reefer_setpoint
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_contract_line_before_insert
BEFORE INSERT ON contract_line
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_consignment_hazard_before_insert
BEFORE INSERT ON consignment_hazard
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_gate_move_before_insert
BEFORE INSERT ON gate_move
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_proof_of_delivery_before_insert
BEFORE INSERT ON proof_of_delivery
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_count_line_before_insert
BEFORE INSERT ON count_line
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_customs_declaration_before_insert
BEFORE INSERT ON customs_declaration
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_transit_guarantee_before_insert
BEFORE INSERT ON transit_guarantee
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_charge_allocation_before_insert
BEFORE INSERT ON charge_allocation
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_accrual_before_insert
BEFORE INSERT ON accrual
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_exchange_rate_before_insert
BEFORE INSERT ON exchange_rate
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_vendor_contract_line_before_insert
BEFORE INSERT ON vendor_contract_line
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_crew_member_before_insert
BEFORE INSERT ON crew_member
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_payroll_line_before_insert
BEFORE INSERT ON payroll_line
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_commercial_invoice_doc_before_insert
BEFORE INSERT ON commercial_invoice_doc
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_claim_document_before_insert
BEFORE INSERT ON claim_document
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trg_service_kpi_before_insert
BEFORE INSERT ON service_kpi
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

-- -----------------------------------------------------------------------------
-- Routines
-- -----------------------------------------------------------------------------

DELIMITER $$

CREATE FUNCTION fn_country_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM country;
  RETURN v_count;
END$$

CREATE FUNCTION fn_tax_jurisdiction_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM tax_jurisdiction;
  RETURN v_count;
END$$

CREATE FUNCTION fn_port_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM port;
  RETURN v_count;
END$$

CREATE FUNCTION fn_customs_office_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM customs_office;
  RETURN v_count;
END$$

CREATE FUNCTION fn_vendor_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM vendor;
  RETURN v_count;
END$$

CREATE FUNCTION fn_vessel_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM vessel;
  RETURN v_count;
END$$

CREATE FUNCTION fn_container_repair_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM container_repair;
  RETURN v_count;
END$$

CREATE FUNCTION fn_surcharge_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM surcharge;
  RETURN v_count;
END$$

CREATE FUNCTION fn_consignment_hazard_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM consignment_hazard;
  RETURN v_count;
END$$

CREATE FUNCTION fn_stowage_plan_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM stowage_plan;
  RETURN v_count;
END$$

CREATE FUNCTION fn_putaway_task_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM putaway_task;
  RETURN v_count;
END$$

CREATE FUNCTION fn_storage_charge_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM storage_charge;
  RETURN v_count;
END$$

CREATE FUNCTION fn_transit_guarantee_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM transit_guarantee;
  RETURN v_count;
END$$

CREATE FUNCTION fn_invoice_line_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM invoice_line;
  RETURN v_count;
END$$

CREATE FUNCTION fn_purchase_invoice_line_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM purchase_invoice_line;
  RETURN v_count;
END$$

CREATE FUNCTION fn_goods_receipt_line_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM goods_receipt_line;
  RETURN v_count;
END$$

CREATE FUNCTION fn_crew_member_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM crew_member;
  RETURN v_count;
END$$

CREATE FUNCTION fn_document_version_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM document_version;
  RETURN v_count;
END$$

CREATE FUNCTION fn_edi_partner_profile_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM edi_partner_profile;
  RETURN v_count;
END$$

CREATE FUNCTION fn_nonconformance_count()
RETURNS BIGINT UNSIGNED
READS SQL DATA
BEGIN
  DECLARE v_count BIGINT UNSIGNED;
  SELECT COUNT(*) INTO v_count FROM nonconformance;
  RETURN v_count;
END$$

CREATE PROCEDURE sp_currency_by_id(IN p_currency_id BIGINT UNSIGNED)
BEGIN
  SELECT currency_id, iso_alpha3, created_at
  FROM currency
  WHERE currency_id = p_currency_id;
END$$

CREATE PROCEDURE sp_document_type_by_id(IN p_document_type_id BIGINT UNSIGNED)
BEGIN
  SELECT document_type_id, description, created_at
  FROM document_type
  WHERE document_type_id = p_document_type_id;
END$$

CREATE PROCEDURE sp_port_terminal_by_id(IN p_port_terminal_id BIGINT UNSIGNED)
BEGIN
  SELECT port_terminal_id, contact_phone, created_at
  FROM port_terminal
  WHERE port_terminal_id = p_port_terminal_id;
END$$

CREATE PROCEDURE sp_free_zone_by_id(IN p_free_zone_id BIGINT UNSIGNED)
BEGIN
  SELECT free_zone_id, contact_phone, created_at
  FROM free_zone
  WHERE free_zone_id = p_free_zone_id;
END$$

CREATE PROCEDURE sp_vendor_rating_by_id(IN p_vendor_rating_id BIGINT UNSIGNED)
BEGIN
  SELECT vendor_rating_id, legal_name, created_at
  FROM vendor_rating
  WHERE vendor_rating_id = p_vendor_rating_id;
END$$

CREATE PROCEDURE sp_vessel_certificate_by_id(IN p_vessel_certificate_id BIGINT UNSIGNED)
BEGIN
  SELECT vessel_certificate_id, flag_state, created_at
  FROM vessel_certificate
  WHERE vessel_certificate_id = p_vessel_certificate_id;
END$$

CREATE PROCEDURE sp_container_lease_by_id(IN p_container_lease_id BIGINT UNSIGNED)
BEGIN
  SELECT container_lease_id, gross_tonnage, created_at
  FROM container_lease
  WHERE container_lease_id = p_container_lease_id;
END$$

CREATE PROCEDURE sp_contract_by_id(IN p_contract_id BIGINT UNSIGNED)
BEGIN
  SELECT contract_id, incoterm_code, created_at
  FROM contract
  WHERE contract_id = p_contract_id;
END$$

CREATE PROCEDURE sp_consignment_temperature_by_id(IN p_consignment_temperature_id BIGINT UNSIGNED)
BEGIN
  SELECT consignment_temperature_id, reference, created_at
  FROM consignment_temperature
  WHERE consignment_temperature_id = p_consignment_temperature_id;
END$$

CREATE PROCEDURE sp_stowage_slot_by_id(IN p_stowage_slot_id BIGINT UNSIGNED)
BEGIN
  SELECT stowage_slot_id, mode_of_transport, created_at
  FROM stowage_slot
  WHERE stowage_slot_id = p_stowage_slot_id;
END$$

CREATE PROCEDURE sp_stock_item_by_id(IN p_stock_item_id BIGINT UNSIGNED)
BEGIN
  SELECT stock_item_id, location_code, created_at
  FROM stock_item
  WHERE stock_item_id = p_stock_item_id;
END$$

CREATE PROCEDURE sp_handling_task_by_id(IN p_handling_task_id BIGINT UNSIGNED)
BEGIN
  SELECT handling_task_id, batch_no, created_at
  FROM handling_task
  WHERE handling_task_id = p_handling_task_id;
END$$

CREATE PROCEDURE sp_sanctions_screening_by_id(IN p_sanctions_screening_id BIGINT UNSIGNED)
BEGIN
  SELECT sanctions_screening_id, regime_code, created_at
  FROM sanctions_screening
  WHERE sanctions_screening_id = p_sanctions_screening_id;
END$$

CREATE PROCEDURE sp_credit_note_by_id(IN p_credit_note_id BIGINT UNSIGNED)
BEGIN
  SELECT credit_note_id, cost_centre, created_at
  FROM credit_note
  WHERE credit_note_id = p_credit_note_id;
END$$

CREATE PROCEDURE sp_disbursement_by_id(IN p_disbursement_id BIGINT UNSIGNED)
BEGIN
  SELECT disbursement_id, cost_centre, created_at
  FROM disbursement
  WHERE disbursement_id = p_disbursement_id;
END$$

CREATE PROCEDURE sp_vendor_contract_by_id(IN p_vendor_contract_id BIGINT UNSIGNED)
BEGIN
  SELECT vendor_contract_id, document_no, created_at
  FROM vendor_contract
  WHERE vendor_contract_id = p_vendor_contract_id;
END$$

CREATE PROCEDURE sp_crew_certificate_by_id(IN p_crew_certificate_id BIGINT UNSIGNED)
BEGIN
  SELECT crew_certificate_id, given_name, created_at
  FROM crew_certificate
  WHERE crew_certificate_id = p_crew_certificate_id;
END$$

CREATE PROCEDURE sp_document_link_by_id(IN p_document_link_id BIGINT UNSIGNED)
BEGIN
  SELECT document_link_id, checksum_sha256, created_at
  FROM document_link
  WHERE document_link_id = p_document_link_id;
END$$

CREATE PROCEDURE sp_edi_error_by_id(IN p_edi_error_id BIGINT UNSIGNED)
BEGIN
  SELECT edi_error_id, checksum_sha256, created_at
  FROM edi_error
  WHERE edi_error_id = p_edi_error_id;
END$$

CREATE PROCEDURE sp_corrective_action_by_id(IN p_corrective_action_id BIGINT UNSIGNED)
BEGIN
  SELECT corrective_action_id, unit_label, created_at
  FROM corrective_action
  WHERE corrective_action_id = p_corrective_action_id;
END$$

DELIMITER ;


-- =============================================================================
-- WL-003 — the small workload, in `freight_wl003`
-- =============================================================================

DROP DATABASE IF EXISTS freight_wl003;
CREATE DATABASE freight_wl003
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_520_ci;
USE freight_wl003;

-- WL-003 is the common path: the read a caller repeats once per object, since
-- FR-RND-002 gives one render per invocation and BR-RND-002 moves iteration to
-- the caller. It is one table, 12 columns and 3 indexes, counted on exactly the
-- rules WL-001 is counted on — the primary key is one of the three indexes.

CREATE TABLE consignment (
  consignment_id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  booking_reference          VARCHAR(32) NOT NULL,
  status                     VARCHAR(24) NOT NULL,
  origin_locode              CHAR(5) NOT NULL,
  destination_locode         CHAR(5) NOT NULL,
  gross_weight_kg            DECIMAL(12,3),
  volume_cbm                 DECIMAL(10,3),
  piece_count                INT UNSIGNED,
  declared_value             DECIMAL(14,2),
  currency_code              CHAR(3),
  created_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (consignment_id),
  UNIQUE KEY ux_consignment_booking_reference (booking_reference),
  KEY idx_consignment_status_created (status, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='One consignment moving under one booking';
