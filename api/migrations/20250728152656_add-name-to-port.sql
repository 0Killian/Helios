-- Add migration script here
alter table core.service_ports add column name text not null default '';
