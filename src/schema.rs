table! {
    actions (id) {
        id -> Int8,
        guild_id -> Int8,
        user_id -> Nullable<Int8>,
        regex -> Nullable<Text>,
        reactions -> Array<Bpchar>,
        expiration -> Nullable<Int8>,
    }
}
