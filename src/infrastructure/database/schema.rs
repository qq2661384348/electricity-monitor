// @generated automatically by Diesel CLI.

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
    }
}
