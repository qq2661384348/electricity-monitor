// @generated automatically by Diesel CLI.

diesel::table! {
    room_paths (id) {
        id -> Uuid,
        roomid -> Int4,
        #[max_length = 255]
        roompath -> Varchar,
        roompath_hash -> Int8,
        #[max_length = 64]
        room_name -> Varchar,
        #[max_length = 50]
        source_type -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    room_sync_log (id) {
        id -> Uuid,
        #[max_length = 50]
        sync_type -> Varchar,
        started_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
        #[max_length = 50]
        status -> Varchar,
        stats -> Nullable<Jsonb>,
        error_message -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    rooms (id) {
        id -> Uuid,
        roomid -> Int4,
        electricity_fee -> Float4,
        send_flag -> Bool,
        threshold -> Float4,
        #[max_length = 64]
        room_name -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        #[max_length = 255]
        primary_roompath -> Varchar,
        primary_roompath_hash -> Int8,
        has_additional_paths -> Bool,
        is_active -> Bool,
        #[max_length = 50]
        source_type -> Varchar,
        #[max_length = 100]
        external_id -> Nullable<Varchar>,
        last_synced_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(room_paths, room_sync_log, rooms,);
