-- Enable required extensions
create extension if not exists pgcrypto;
create extension if not exists ltree;

-- 2. Profiles Table (Extends Supabase Auth users)
create table public.profiles (
    id uuid references auth.users on delete cascade primary key,
    updated_at timestamp with time zone default timezone('utc'::text, now()) not null,
    full_name text,
    avatar_url text,
    storage_quota_bytes bigint default 5368709120 not null, -- Default 5GB
    storage_used_bytes bigint default 0 not null,
    constraint storage_used_positive check (storage_used_bytes >= 0)
);

-- 3. Physical Storage Objects (Deduplicated files repository)
create table public.storage_objects (
    id uuid default gen_random_uuid() primary key,
    sha256_hash char(64) not null unique,
    storage_path text not null unique,
    size_bytes bigint not null,
    mime_type text not null,
    compression_type text default 'NONE'::text not null, -- 'NONE', 'ZSTD'
    reference_count integer default 1 not null,
    created_at timestamp with time zone default timezone('utc'::text, now()) not null,
    constraint size_positive check (size_bytes > 0),
    constraint ref_count_positive check (reference_count >= 0)
);

-- 4. Folders Table (Materialized Path pattern)
create table public.folders (
    id uuid default gen_random_uuid() primary key,
    user_id uuid references public.profiles(id) on delete cascade not null,
    name text not null,
    parent_id uuid references public.folders(id) on delete cascade,
    path ltree,
    created_at timestamp with time zone default timezone('utc'::text, now()) not null,
    constraint folder_name_validation check (length(name) >= 1 and length(name) <= 255)
);

-- 5. User Files Table (User metadata pointers to deduplicated objects)
create table public.files (
    id uuid default gen_random_uuid() primary key,
    user_id uuid references public.profiles(id) on delete cascade not null,
    folder_id uuid references public.folders(id) on delete set null,
    storage_object_id uuid references public.storage_objects(id) on delete restrict,
    display_name text not null,
    is_favorite boolean default false not null,
    status text default 'PENDING'::text not null, -- 'PENDING', 'PROCESSING', 'ACTIVE', 'QUARANTINED'
    created_at timestamp with time zone default timezone('utc'::text, now()) not null,
    updated_at timestamp with time zone default timezone('utc'::text, now()) not null,
    constraint file_name_validation check (length(display_name) >= 1 and length(display_name) <= 255),
    constraint status_enum check (status in ('PENDING', 'PROCESSING', 'ACTIVE', 'QUARANTINED'))
);

-- 6. Audit Logging Table
create table public.audit_logs (
    id uuid default gen_random_uuid() primary key,
    user_id uuid references public.profiles(id) on delete set null,
    action text not null, -- 'UPLOAD', 'DOWNLOAD', 'DELETE', 'SHARE'
    file_id uuid,
    ip_address inet,
    user_agent text,
    details jsonb,
    created_at timestamp with time zone default timezone('utc'::text, now()) not null
);

-- 7. High-Performance Indexes
create index idx_files_user_id on public.files(user_id);
create index idx_files_storage_object on public.files(storage_object_id);
create index idx_folders_path on public.folders using gist(path);
create index idx_storage_objects_hash on public.storage_objects(sha256_hash);
create index idx_audit_logs_user_action on public.audit_logs(user_id, action);

-- 8. Enable Row-Level Security (RLS)
alter table public.profiles enable row level security;
alter table public.folders enable row level security;
alter table public.files enable row level security;
alter table public.audit_logs enable row level security;

-- 9. Row-Level Security Isolation Policies
create policy "Users can view own profile" on public.profiles
    for select using (auth.uid() = id);

create policy "Users can update own profile" on public.profiles
    for update using (auth.uid() = id);

create policy "Folders tenant isolation" on public.folders
    for all using (auth.uid() = user_id);

create policy "Files tenant isolation" on public.files
    for all using (auth.uid() = user_id);

create policy "Audit logs tenant isolation" on public.audit_logs
    for select using (auth.uid() = user_id);

-- 10. Automated Profile Creation Trigger
-- When a user registers via Supabase Auth, automatically instantiate their Profile
create or replace function public.handle_new_user()
returns trigger as $$
begin
  insert into public.profiles (id, full_name, avatar_url)
  values (
    new.id,
    coalesce(new.raw_user_meta_data->>'full_name', ''),
    coalesce(new.raw_user_meta_data->>'avatar_url', '')
  );
  return new;
end;
$$ language plpgsql security definer;

create trigger on_auth_user_created
  after insert on auth.users
  for each row execute procedure public.handle_new_user();