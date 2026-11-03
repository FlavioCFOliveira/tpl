-- The main SQLite fixture database of FR-LITE-038: a freight operator's
-- schedule, carriers, consignments and customs records. It is built from this
-- text at test time, by tests/support/sqlite.rs, through the SQLite library
-- the binary carries; no binary database file is committed.
--
-- The numbered comments name the item of FR-LITE-038 each object exercises.
-- The builder runs this text on a plain connection with none of tpl's
-- controls, so that item 9 is written under trusted_schema on.

-- The library the binary carries enforces foreign keys by default, and a key
-- naming a table the database does not hold (item 8) refuses every insert into
-- its table while enforcement is on; the population below needs it off.
PRAGMA foreign_keys = OFF;

-- Item 1: ordinary rowid tables. Item 6: a UNIQUE constraint at table level,
-- and a collated column.
CREATE TABLE country (
    country_code TEXT PRIMARY KEY,
    country_name VARCHAR(80) NOT NULL COLLATE NOCASE,
    UNIQUE (country_name)
);

-- Item 8: a key naming no parent column, with CASCADE on update and RESTRICT
-- on delete.
CREATE TABLE port (
    port_code CHAR(5) PRIMARY KEY,
    port_name TEXT NOT NULL,
    country_code TEXT NOT NULL REFERENCES country ON UPDATE CASCADE ON DELETE RESTRICT,
    latitude REAL,
    longitude DOUBLE PRECISION
);

-- Item 4: a rowid alias with AUTOINCREMENT, which creates sqlite_sequence
-- (item 12). Item 6: UNIQUE at column level. Items 2 and 3: the affinities
-- and the default rows. Item 9: a CHECK at table level.
CREATE TABLE carrier (
    carrier_id INTEGER PRIMARY KEY AUTOINCREMENT,
    scac_code VARCHAR(4) NOT NULL UNIQUE,
    legal_name TEXT NOT NULL DEFAULT 'unnamed',
    founded DATETIME,
    rating NUMERIC(5,2) DEFAULT (4.5),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    notes,
    logo BLOB DEFAULT x'00ff',
    CHECK (rating IS NULL OR rating BETWEEN 0 AND 5)
);

-- Item 4: a rowid alias without AUTOINCREMENT. Item 9: a CHECK at column
-- level, and a named one. Item 8: SET NULL on update and on delete, and a key
-- declared MATCH FULL with NO ACTION.
CREATE TABLE vessel (
    vessel_imo INTEGER PRIMARY KEY,
    vessel_name TEXT NOT NULL CHECK (length(vessel_name) > 0),
    carrier_id INTEGER REFERENCES carrier (carrier_id) ON UPDATE SET NULL ON DELETE SET NULL,
    gross_tonnage INT,
    flag_country TEXT REFERENCES country (country_code) MATCH FULL ON UPDATE NO ACTION ON DELETE NO ACTION,
    deadweight_tonnes REAL,
    CONSTRAINT ck_vessel_tonnage CHECK (gross_tonnage IS NULL OR gross_tonnage > 0)
);

-- Item 4: the four integer primary keys FR-LITE-028 observes as no alias.
CREATE TABLE berth (
    berth_no INT PRIMARY KEY,
    port_code CHAR(5) NOT NULL REFERENCES port (port_code) ON UPDATE RESTRICT ON DELETE CASCADE,
    max_draught_m REAL
);

CREATE TABLE container_size (
    size_code BIGINT PRIMARY KEY,
    teu REAL NOT NULL DEFAULT 1.0,
    description TEXT
);

CREATE TABLE tariff_band (
    band_id INTEGER PRIMARY KEY DESC,
    rate_per_teu DECIMAL(12,3) NOT NULL DEFAULT 0.0,
    currency CHAR(3) NOT NULL DEFAULT 'EUR'
);

CREATE TABLE hazard_class (
    class_id INTEGER PRIMARY KEY,
    label TEXT NOT NULL
) WITHOUT ROWID;

-- Item 5: a composite primary key. Item 8: a key naming no parent column on
-- a rowid-alias parent, and SET DEFAULT on update and on delete.
CREATE TABLE voyage (
    vessel_imo INTEGER NOT NULL REFERENCES vessel ON UPDATE CASCADE ON DELETE CASCADE,
    voyage_no INTEGER NOT NULL,
    departs_on TEXT DEFAULT CURRENT_TIMESTAMP,
    origin_port CHAR(5) DEFAULT 'XXXXX' REFERENCES port (port_code) ON UPDATE SET DEFAULT ON DELETE SET DEFAULT,
    status TEXT NOT NULL DEFAULT 'planned',
    PRIMARY KEY (vessel_imo, voyage_no)
);

-- Item 1: a WITHOUT ROWID table. Item 7: a virtual and a stored generated
-- column. Item 8: a composite key.
CREATE TABLE voyage_leg (
    vessel_imo INTEGER NOT NULL,
    voyage_no INTEGER NOT NULL,
    leg_seq INTEGER NOT NULL,
    from_port CHAR(5) NOT NULL REFERENCES port (port_code),
    to_port CHAR(5) NOT NULL REFERENCES port (port_code),
    distance_nm REAL,
    eta_hours REAL GENERATED ALWAYS AS (distance_nm / 14.0) VIRTUAL,
    distance_km REAL GENERATED ALWAYS AS (distance_nm * 1.852) STORED,
    PRIMARY KEY (vessel_imo, voyage_no, leg_seq),
    FOREIGN KEY (vessel_imo, voyage_no) REFERENCES voyage (vessel_imo, voyage_no) ON DELETE CASCADE
) WITHOUT ROWID;

-- Item 3: every row of the default classification, on nullable and NOT NULL
-- columns. Item 8: a self-reference, and a key naming a table the database
-- does not hold.
CREATE TABLE consignment (
    consignment_id INTEGER PRIMARY KEY,
    reference TEXT NOT NULL,
    shipper TEXT DEFAULT 'Lloyd''s "A1"',
    parent_consignment_id INTEGER REFERENCES consignment (consignment_id) ON DELETE SET NULL,
    customs_office_id INTEGER REFERENCES customs_office (office_id),
    booked_at TEXT DEFAULT (datetime('now')),
    insured BOOLEAN NOT NULL DEFAULT false,
    priority INTEGER DEFAULT +5,
    ratio REAL DEFAULT 1e3,
    hex_flags INTEGER DEFAULT 0x1F,
    empty_note TEXT DEFAULT '',
    summed INTEGER DEFAULT (1+2),
    lowered TEXT DEFAULT (  lower( 'X' )  ),
    negative REAL DEFAULT (-1),
    cleared TEXT DEFAULT NULL,
    required_note TEXT NOT NULL DEFAULT NULL,
    signature BLOB DEFAULT X'AB',
    received_at TEXT DEFAULT current_time,
    free_text TEXT,
    seal_code TEXT NOT NULL
);

-- Item 1: a STRICT table. Item 2: a STRICT column declared ANY.
CREATE TABLE manifest_line (
    line_id INTEGER PRIMARY KEY,
    consignment_id INTEGER NOT NULL REFERENCES consignment (consignment_id) ON DELETE CASCADE,
    payload ANY,
    weight_kg REAL NOT NULL DEFAULT 0.0,
    declared_value INTEGER DEFAULT -1,
    hs_code TEXT DEFAULT NULL
) STRICT;

-- Item 9: a CHECK calling a function the carried library holds and does not
-- mark innocuous, written under trusted_schema on.
CREATE TABLE inspection_zone (
    zone_id INTEGER PRIMARY KEY,
    zone_name TEXT NOT NULL,
    zone_shape BLOB,
    CHECK (zone_shape IS NULL OR rtreedepth(zone_shape) >= 0)
);

-- Item 13: identifiers carrying a space, a backtick and a double quote.
CREATE TABLE "customs entry" (
    entry_id INTEGER PRIMARY KEY,
    `officer``s stamp` TEXT,
    """seal"" no" TEXT NOT NULL,
    consignment_id INTEGER REFERENCES consignment (consignment_id)
);

-- Item 12: a virtual table with shadow tables.
CREATE VIRTUAL TABLE cargo_note USING fts5 (consignment_ref, body);

-- Item 6: CREATE INDEX unique and not, a descending member, a collated member,
-- a partial index, an expression index, and an index on a generated column.
CREATE UNIQUE INDEX ux_vessel_name ON vessel (vessel_name COLLATE NOCASE);
CREATE INDEX ix_voyage_departs ON voyage (departs_on DESC, status);
CREATE INDEX ix_consignment_uninsured ON consignment (booked_at) WHERE insured = 0;
CREATE INDEX ix_port_name_lower ON port (lower(port_name));
CREATE INDEX ix_leg_eta ON voyage_leg (eta_hours);
CREATE INDEX "ix customs entry" ON "customs entry" (`officer``s stamp`);

-- Item 10: views, one with a column list, one selecting from a missing table,
-- one calling a function the carried library does not hold, and one calling a
-- function it holds and does not mark innocuous.
CREATE VIEW v_vessel_fleet AS
    SELECT v.vessel_imo, v.vessel_name, c.legal_name AS carrier_name
    FROM vessel AS v LEFT JOIN carrier AS c ON c.carrier_id = v.carrier_id;

CREATE VIEW v_port_country (port, country) AS
    SELECT p.port_name, c.country_name FROM port AS p JOIN country AS c USING (country_code);

CREATE VIEW v_quay_schedule AS SELECT * FROM quay_schedule;

CREATE VIEW v_document_paths AS SELECT realpath(reference) AS location FROM consignment;

CREATE VIEW v_ranked_notes AS
    SELECT consignment_ref, bm25(cargo_note) AS score FROM cargo_note WHERE cargo_note MATCH 'reefer';

-- Item 11: triggers of each timing on each event, one with no timing written,
-- one UPDATE OF a column list, one with a WHEN clause, one with a quoted name,
-- and an INSTEAD OF trigger on a view.
CREATE TABLE voyage_audit (
    audit_id INTEGER PRIMARY KEY,
    vessel_imo INTEGER,
    voyage_no INTEGER,
    action TEXT NOT NULL
);

CREATE TRIGGER trg_voyage_before_insert BEFORE INSERT ON voyage
BEGIN
    SELECT RAISE(ABORT, 'a voyage needs a vessel') WHERE NEW.vessel_imo IS NULL;
END;

CREATE TRIGGER trg_voyage_after_insert AFTER INSERT ON voyage
BEGIN
    INSERT INTO voyage_audit (vessel_imo, voyage_no, action) VALUES (NEW.vessel_imo, NEW.voyage_no, 'insert');
END;

CREATE TRIGGER trg_voyage_before_update BEFORE UPDATE OF status, departs_on ON voyage
BEGIN
    SELECT RAISE(ABORT, 'a sailed voyage is closed') WHERE OLD.status = 'sailed';
END;

CREATE TRIGGER trg_voyage_after_update after update ON voyage
WHEN NEW.status = 'sailed'
BEGIN
    INSERT INTO voyage_audit (vessel_imo, voyage_no, action) VALUES (NEW.vessel_imo, NEW.voyage_no, 'sailed');
END;

CREATE TRIGGER trg_voyage_before_delete BEFORE DELETE ON voyage
BEGIN
    SELECT RAISE(ABORT, 'a voyage with legs is kept')
    WHERE EXISTS (SELECT 1 FROM voyage_leg WHERE vessel_imo = OLD.vessel_imo AND voyage_no = OLD.voyage_no);
END;

CREATE TRIGGER "trg voyage ""after"" delete" AFTER DELETE ON voyage
BEGIN
    INSERT INTO voyage_audit (vessel_imo, voyage_no, action) VALUES (OLD.vessel_imo, OLD.voyage_no, 'delete');
END;

CREATE TRIGGER trg_port_default_timing INSERT ON port
BEGIN
    SELECT RAISE(ABORT, 'a port code has five characters') WHERE length(NEW.port_code) <> 5;
END;

CREATE TRIGGER trg_fleet_rename INSTEAD OF UPDATE OF vessel_name ON v_vessel_fleet
BEGIN
    UPDATE vessel SET vessel_name = NEW.vessel_name WHERE vessel_imo = OLD.vessel_imo;
END;

-- A realistic population, so that sqlite_sequence holds a row and ANALYZE has
-- something to record in sqlite_stat1 (item 12).
INSERT INTO country (country_code, country_name) VALUES
    ('PT', 'Portugal'),
    ('NL', 'Netherlands'),
    ('SG', 'Singapore');

INSERT INTO port (port_code, port_name, country_code, latitude, longitude) VALUES
    ('PTLIS', 'Lisbon', 'PT', 38.7071, -9.1355),
    ('PTSIE', 'Sines', 'PT', 37.9500, -8.8667),
    ('NLRTM', 'Rotterdam', 'NL', 51.9496, 4.1453),
    ('SGSIN', 'Singapore', 'SG', 1.2644, 103.8222),
    ('XXXXX', 'Unassigned', 'PT', NULL, NULL);

INSERT INTO carrier (scac_code, legal_name, founded, rating, is_active) VALUES
    ('MAEU', 'Maersk Line', '1904-04-16', 4.4, TRUE),
    ('MSCU', 'Mediterranean Shipping Company', '1970-10-01', 4.2, TRUE);

INSERT INTO vessel (vessel_imo, vessel_name, carrier_id, gross_tonnage, flag_country, deadweight_tonnes) VALUES
    (9778791, 'Madrid Maersk', 1, 214286, 'NL', 192672),
    (9839131, 'MSC Gulsun', 2, 232618, 'PT', 197362);

INSERT INTO voyage (vessel_imo, voyage_no, departs_on, origin_port, status) VALUES
    (9778791, 2611, '2026-11-02 06:00:00', 'NLRTM', 'planned'),
    (9839131, 2614, '2026-11-05 18:30:00', 'PTSIE', 'planned');

INSERT INTO voyage_leg (vessel_imo, voyage_no, leg_seq, from_port, to_port, distance_nm) VALUES
    (9778791, 2611, 1, 'NLRTM', 'PTSIE', 1370),
    (9778791, 2611, 2, 'PTSIE', 'SGSIN', 8310);

INSERT INTO consignment (consignment_id, reference, shipper, insured, seal_code, required_note) VALUES
    (40211, 'BKG-2026-40211', 'Lusoexport, Lda.', TRUE, 'SL-778201', 'reefer, set point 4 C'),
    (40212, 'BKG-2026-40212', 'Lloyd''s "A1" Brokers', FALSE, 'SL-778202', 'dry van');

INSERT INTO manifest_line (consignment_id, payload, weight_kg, declared_value, hs_code) VALUES
    (40211, 'Rocha pears, 22 pallets', 18450.0, 42000, '0808.30'),
    (40212, x'4d414e4946455354', 9120.5, 15800, '6403.99');

INSERT INTO cargo_note (consignment_ref, body) VALUES
    ('BKG-2026-40211', 'reefer container, pre-cooled to 4 C before stuffing'),
    ('BKG-2026-40212', 'dry van, fumigation certificate attached');

ANALYZE;
