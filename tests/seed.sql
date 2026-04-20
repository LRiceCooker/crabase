-- Seed SQL for crabase integration tests
-- Covers all Postgres types the app supports

-- Clean up if re-running
DROP SCHEMA IF EXISTS test_schema CASCADE;
DROP TABLE IF EXISTS public.users CASCADE;
DROP TABLE IF EXISTS public.products CASCADE;
DROP TABLE IF EXISTS public.events CASCADE;
DROP TYPE IF EXISTS public.user_role CASCADE;
DROP TYPE IF EXISTS public.priority_level CASCADE;

-- Custom enum types
CREATE TYPE public.user_role AS ENUM ('admin', 'editor', 'viewer', 'guest');
CREATE TYPE public.priority_level AS ENUM ('low', 'medium', 'high', 'critical');

-- Table 1: users — covers text, integer, boolean, enum, timestamp, uuid, json, array
CREATE TABLE public.users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) NOT NULL,
    email TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'viewer',
    is_active BOOLEAN NOT NULL DEFAULT true,
    age SMALLINT,
    login_count INTEGER DEFAULT 0,
    score BIGINT,
    rating NUMERIC(5, 2),
    balance MONEY,
    uuid UUID DEFAULT gen_random_uuid(),
    tags TEXT[] DEFAULT '{}',
    metadata JSONB,
    preferences JSON,
    bio TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table 2: products — covers float, double, date, time, interval, bytea, bit, inet, macaddr, xml
CREATE TABLE public.products (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price FLOAT4 NOT NULL,
    weight FLOAT8,
    release_date DATE,
    available_from TIME,
    available_until TIMETZ,
    warranty_period INTERVAL,
    priority priority_level DEFAULT 'medium',
    sku BIT(8),
    flags BIT VARYING(16),
    manufacturer_ip INET,
    device_mac MACADDR,
    description XML,
    thumbnail BYTEA,
    int_range INT4RANGE,
    date_range DATERANGE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Table 3: events — covers timestamptz, point, cidr, arrays of various types
CREATE TABLE public.events (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    scheduled_at TIMESTAMPTZ NOT NULL,
    duration INTERVAL,
    location POINT,
    network CIDR,
    attendee_ids INTEGER[],
    categories TEXT[],
    scores NUMERIC(5,2)[],
    is_public BOOLEAN DEFAULT true,
    extra JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert 12 rows into users
INSERT INTO public.users (username, email, role, is_active, age, login_count, score, rating, balance, tags, metadata, preferences, bio, created_at, updated_at) VALUES
('alice', 'alice@example.com', 'admin', true, 30, 150, 99999, 4.85, '$1000.00', ARRAY['staff', 'dev'], '{"level": 10, "verified": true}', '{"theme": "dark"}', 'Software engineer', '2024-01-01 10:00:00', '2024-06-15 12:00:00+00'),
('bob', 'bob@example.com', 'editor', true, 25, 42, 5000, 3.50, '$250.00', ARRAY['writer'], '{"level": 5}', '{"theme": "light"}', 'Content writer', '2024-01-15 09:00:00', '2024-06-10 08:30:00+00'),
('charlie', 'charlie@example.com', 'viewer', false, 45, 3, 100, 2.10, '$10.00', ARRAY[]::TEXT[], '{}', NULL, NULL, '2024-02-01 14:00:00', '2024-03-01 14:00:00+00'),
('diana', 'diana@example.com', 'admin', true, 35, 200, 150000, 4.99, '$5000.00', ARRAY['staff', 'lead', 'dev'], '{"level": 15, "verified": true, "badges": ["gold", "platinum"]}', '{"theme": "dark", "notifications": true}', 'Engineering lead', '2024-02-10 08:00:00', '2024-07-01 09:00:00+00'),
('eve', 'eve@example.com', 'guest', true, NULL, 0, NULL, NULL, NULL, NULL, NULL, NULL, NULL, '2024-03-01 12:00:00', '2024-03-01 12:00:00+00'),
('frank', 'frank@example.com', 'editor', true, 28, 75, 8500, 3.80, '$420.00', ARRAY['designer'], '{"level": 7}', '{"theme": "light", "sidebar": false}', 'UI designer', '2024-03-15 11:00:00', '2024-05-20 16:00:00+00'),
('grace', 'grace@example.com', 'viewer', true, 22, 10, 500, 2.50, '$30.00', ARRAY['intern'], '{"level": 1}', NULL, 'Summer intern', '2024-04-01 09:30:00', '2024-04-01 09:30:00+00'),
('heidi', 'heidi@example.com', 'admin', true, 40, 300, 250000, 5.00, '$10000.00', ARRAY['staff', 'exec'], '{"level": 20, "verified": true}', '{"theme": "dark"}', 'CTO', '2023-06-01 08:00:00', '2024-07-15 10:00:00+00'),
('ivan', 'ivan@example.com', 'editor', false, 33, 55, 6000, 3.20, '$180.00', ARRAY['writer', 'reviewer'], '{"level": 6}', '{"notifications": false}', 'Technical writer', '2024-04-15 13:00:00', '2024-06-01 11:00:00+00'),
('judy', 'judy@example.com', 'viewer', true, 27, 20, 1500, 3.00, '$75.00', ARRAY['tester'], '{"level": 3}', NULL, 'QA engineer', '2024-05-01 10:00:00', '2024-05-01 10:00:00+00'),
('karl', 'karl@example.com', 'guest', true, 50, 1, 10, 1.00, '$1.00', ARRAY[]::TEXT[], '{}', NULL, NULL, '2024-05-15 14:00:00', '2024-05-15 14:00:00+00'),
('lara', 'lara@example.com', 'editor', true, 31, 90, 12000, 4.20, '$600.00', ARRAY['dev', 'writer'], '{"level": 8, "verified": true}', '{"theme": "dark", "editor_font_size": 14}', 'Full-stack dev', '2024-06-01 08:00:00', '2024-07-10 09:30:00+00');

-- Insert 12 rows into products
INSERT INTO public.products (name, price, weight, release_date, available_from, available_until, warranty_period, priority, sku, flags, manufacturer_ip, device_mac, description, thumbnail, int_range, date_range, created_at) VALUES
('Widget A', 9.99, 0.5, '2024-01-15', '08:00:00', '17:00:00+00', '1 year', 'high', B'10101010', B'1100110011001100', '192.168.1.1', '08:00:2b:01:02:03', '<product><name>Widget A</name></product>', E'\\xDEADBEEF', '[1, 100]', '[2024-01-01, 2024-12-31]', '2024-01-15 10:00:00'),
('Gadget B', 29.99, 1.2, '2024-02-20', '09:00:00', '18:00:00+05:30', '2 years', 'medium', B'11001100', B'1010', '10.0.0.1', '08:00:2b:04:05:06', '<product><name>Gadget B</name></product>', E'\\xCAFEBABE', '[10, 500]', '[2024-03-01, 2025-03-01]', '2024-02-20 14:00:00'),
('Doohickey C', 4.50, 0.1, '2024-03-10', '00:00:00', '23:59:59+00', '6 months', 'low', B'00001111', NULL, '172.16.0.1', NULL, NULL, NULL, NULL, NULL, '2024-03-10 09:00:00'),
('Thingamajig D', 149.99, 5.0, '2024-04-01', '10:00:00', '16:00:00-05:00', '3 years', 'critical', B'11111111', B'0000000000000001', '192.168.0.100', '08:00:2b:07:08:09', '<product><name>Thingamajig D</name><category>premium</category></product>', E'\\x0102030405', '[1, 1000000]', '[2024-01-01, 2027-01-01]', '2024-04-01 08:00:00'),
('Whatchamacallit E', 0.99, 0.01, NULL, NULL, NULL, NULL, 'low', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, '2024-04-15 12:00:00'),
('Gizmo F', 59.99, 2.3, '2024-05-01', '07:00:00', '19:00:00+00', '1 year 6 months', 'high', B'01010101', B'11111111', '10.10.10.10', '08:00:2b:0a:0b:0c', '<product><name>Gizmo F</name></product>', E'\\xFEEDFACE', '[5, 50]', '[2024-05-01, 2024-11-01]', '2024-05-01 11:00:00'),
('Contraption G', 199.99, 8.5, '2024-06-15', '06:00:00', '22:00:00+09:00', '5 years', 'critical', B'11110000', B'1010101010101010', '192.168.100.1', '08:00:2b:0d:0e:0f', '<product><name>Contraption G</name></product>', E'\\x00FF00FF', '[100, 10000]', '[2024-06-15, 2029-06-15]', '2024-06-15 07:00:00'),
('Apparatus H', 34.50, 1.8, '2024-07-01', '08:30:00', '17:30:00+00', '2 years', 'medium', B'10001000', NULL, '10.0.1.1', '08:00:2b:10:11:12', NULL, NULL, '[1, 10]', '[2024-07-01, 2026-07-01]', '2024-07-01 13:00:00'),
('Mechanism I', 75.00, 3.0, '2024-07-20', '09:00:00', '18:00:00+00', '1 year', 'medium', B'01100110', B'110011', '172.16.1.1', NULL, '<product><name>Mechanism I</name></product>', E'\\xABCDEF01', '[50, 500]', '[2024-07-20, 2025-07-20]', '2024-07-20 10:00:00'),
('Device J', 499.99, 12.0, '2024-08-01', '10:00:00', '20:00:00+00', '4 years', 'high', B'11111110', B'1111111111111111', '192.168.50.50', '08:00:2b:13:14:15', '<product><name>Device J</name><tier>enterprise</tier></product>', E'\\x1234567890ABCDEF', '[1000, 100000]', '[2024-08-01, 2028-08-01]', '2024-08-01 09:00:00'),
('Tool K', 15.00, 0.8, '2024-08-15', '07:00:00', '15:00:00+00', '6 months', 'low', B'00110011', NULL, NULL, NULL, NULL, NULL, NULL, NULL, '2024-08-15 14:00:00'),
('Instrument L', 89.99, 2.0, '2024-09-01', '08:00:00', '16:00:00+00', '2 years 6 months', 'high', B'10101001', B'10100101', '10.1.1.1', '08:00:2b:16:17:18', '<product><name>Instrument L</name></product>', E'\\xFACEFEED', '[10, 1000]', '[2024-09-01, 2027-03-01]', '2024-09-01 08:30:00');

-- Insert 12 rows into events
INSERT INTO public.events (title, description, scheduled_at, duration, location, network, attendee_ids, categories, scores, is_public, extra, created_at) VALUES
('Team Standup', 'Daily standup meeting', '2024-07-01 09:00:00+00', '15 minutes', POINT(37.7749, -122.4194), '192.168.1.0/24', ARRAY[1,2,3,4], ARRAY['meeting', 'daily'], ARRAY[4.5, 3.8, 5.0], true, '{"room": "A101", "recurring": true}', '2024-06-30 08:00:00+00'),
('Product Launch', 'Q3 product launch event', '2024-08-15 14:00:00+00', '2 hours', POINT(40.7128, -74.0060), '10.0.0.0/8', ARRAY[1,4,5,8], ARRAY['event', 'marketing', 'product'], ARRAY[5.0, 4.9, 4.8, 5.0], true, '{"venue": "Grand Hall", "capacity": 500}', '2024-07-01 10:00:00+00'),
('Security Review', 'Quarterly security audit', '2024-09-01 10:00:00+00', '3 hours', NULL, '172.16.0.0/12', ARRAY[4,8], ARRAY['security', 'audit'], ARRAY[4.2, 4.0], false, '{"classification": "confidential"}', '2024-08-01 09:00:00+00'),
('Hackathon', '48-hour coding challenge', '2024-10-01 18:00:00+00', '2 days', POINT(51.5074, -0.1278), '192.168.0.0/16', ARRAY[1,2,3,6,7,10,12], ARRAY['event', 'engineering', 'fun'], ARRAY[4.9, 4.7, 5.0, 4.8], true, '{"prizes": ["laptop", "headphones", "gift card"], "teams": 12}', '2024-09-01 12:00:00+00'),
('Board Meeting', NULL, '2024-07-15 16:00:00+00', '1 hour 30 minutes', POINT(37.7749, -122.4194), NULL, ARRAY[4,8], ARRAY['meeting', 'executive'], NULL, false, '{}', '2024-07-10 08:00:00+00'),
('Training: Rust Basics', 'Introduction to Rust for the team', '2024-08-01 13:00:00+00', '4 hours', POINT(37.7749, -122.4194), '192.168.1.0/24', ARRAY[2,3,6,7,9,10], ARRAY['training', 'engineering'], ARRAY[4.6, 4.3, 4.8, 4.5, 4.7, 4.4], true, '{"instructor": "diana", "materials": "https://internal.wiki/rust"}', '2024-07-20 11:00:00+00'),
('Sprint Retro', 'Sprint 14 retrospective', '2024-07-22 15:00:00+00', '1 hour', NULL, NULL, ARRAY[1,2,3,6,7], ARRAY['meeting', 'agile'], ARRAY[3.5, 4.0, 3.8], true, '{"sprint": 14}', '2024-07-21 09:00:00+00'),
('Client Demo', 'Demo new features to Acme Corp', '2024-08-20 11:00:00-05:00', '45 minutes', POINT(41.8781, -87.6298), '10.10.0.0/16', ARRAY[1,4,6], ARRAY['meeting', 'sales', 'demo'], ARRAY[4.8, 4.9], false, '{"client": "Acme Corp", "deal_size": 50000}', '2024-08-10 14:00:00+00'),
('All Hands', 'Monthly all-hands meeting', '2024-08-05 10:00:00+00', '1 hour', POINT(37.7749, -122.4194), '192.168.0.0/16', ARRAY[1,2,3,4,5,6,7,8,9,10,11,12], ARRAY['meeting', 'company'], ARRAY[4.0, 3.5, 4.2], true, '{"presenter": "heidi", "recording": true}', '2024-08-01 08:00:00+00'),
('Deploy Window', 'Production deployment window', '2024-09-10 02:00:00+00', '4 hours', NULL, '10.0.0.0/8', ARRAY[1,4,12], ARRAY['ops', 'deployment'], NULL, false, '{"version": "2.4.0", "rollback_plan": true}', '2024-09-05 16:00:00+00'),
('Holiday Party', 'End of year celebration', '2024-12-20 18:00:00+00', '5 hours', POINT(37.7849, -122.4094), NULL, ARRAY[1,2,3,4,5,6,7,8,9,10,11,12], ARRAY['event', 'social', 'fun'], ARRAY[5.0, 5.0, 4.9, 5.0], true, '{"budget": 10000, "catering": true, "dj": true}', '2024-11-01 09:00:00+00'),
('Incident Post-mortem', 'Analysis of the Sept 5 outage', '2024-09-12 14:00:00+00', '2 hours', NULL, NULL, ARRAY[1,4,8,12], ARRAY['meeting', 'ops', 'incident'], ARRAY[3.0, 2.5], false, '{"incident_id": "INC-2024-091", "severity": "P1", "duration_minutes": 45}', '2024-09-06 10:00:00+00');

-- Non-public schema with its own enum (for testing schema-prefixed enums)
CREATE SCHEMA test_schema;

CREATE TYPE test_schema.status_type AS ENUM ('pending', 'active', 'completed', 'cancelled');

CREATE TABLE test_schema.tasks (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    status test_schema.status_type NOT NULL DEFAULT 'pending',
    assigned_to INTEGER,
    due_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO test_schema.tasks (title, status, assigned_to, due_date, created_at) VALUES
('Setup CI/CD', 'completed', 1, '2024-07-01', '2024-06-15 10:00:00+00'),
('Write tests', 'active', 2, '2024-07-15', '2024-07-01 09:00:00+00'),
('Deploy v2', 'pending', 4, '2024-08-01', '2024-07-10 14:00:00+00'),
('Fix login bug', 'completed', 3, '2024-06-20', '2024-06-18 08:00:00+00'),
('Update docs', 'active', 6, '2024-07-30', '2024-07-05 11:00:00+00'),
('Security patch', 'pending', 8, '2024-08-15', '2024-07-20 16:00:00+00'),
('Refactor DB layer', 'active', 12, '2024-08-10', '2024-07-15 09:30:00+00'),
('Add dark mode', 'completed', 6, '2024-07-10', '2024-06-25 10:00:00+00'),
('Performance audit', 'pending', 4, '2024-09-01', '2024-08-01 08:00:00+00'),
('Onboard new hire', 'cancelled', 1, '2024-07-05', '2024-07-01 12:00:00+00'),
('Migrate to v2 API', 'active', 1, '2024-08-20', '2024-07-25 13:00:00+00'),
('Load testing', 'pending', 12, '2024-09-15', '2024-08-10 10:00:00+00');
