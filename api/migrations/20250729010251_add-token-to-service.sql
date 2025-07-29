-- Add migration script here
alter table core.services add column token text not null;
