-- Add migration script here
alter table core.devices add column last_scanned timestamptz not null default now();
