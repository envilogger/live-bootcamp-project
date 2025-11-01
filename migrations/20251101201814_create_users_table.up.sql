create table if not exists users (
  email varchar(255) primary key,
  password_hash varchar(255) not null,
  requires_2fa boolean not null default false,
  created_at timestamp with time zone not null default(now() at time zone 'utc')
);
