-- Add migration script here

create schema core;

-- Stores the devices connected on the network
create table core.devices (
    mac_address macaddr primary key,
    last_known_ip inet,
    display_name varchar(255) not null,
    is_name_custom boolean not null default false, -- true if the name is user-defined, false if the hostname of the device
    notes text, -- custom notes left by the user
    is_online boolean not null default false, -- checked periodically
    created_at timestamptz with time zone default now(),
    updated_at timestamptz with time zone default now()
);

-- Stores the abstract service
create table core.services (
    id uuid primary key default uuid_generate_v7(),
    device_mac macaddr not null references core.devices(mac_address) on delete cascade,
    display_name varchar(255) not null, -- user-defined name for the service
    kind varchar(255) not null, -- type of service (e.g., DNS, Web, SSH, Mail, ...)
    is_managed boolean not null default false, -- is the service managed by Helios
    created_at timestamptz with time zone default now(),
    updated_at timestamptz with time zone default now()
);

-- This table links a service to its specific network ports
create table core.service_ports (
    service_id uuid references core.services(id) on delete cascade,
    port integer not null,
    transport_protocol varchar(3) not null check(transport_protocol in ('TCP', 'UDP')),
    application_protocol varchar(255) not null,
    is_online boolean not null default false, -- checked periodically
    created_at timestamptz with time zone default now(),
    updated_at timestamptz with time zone default now(),

    primary key (service_id, port, transport_protocol)
);
