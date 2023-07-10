package com.stuartharris.carbon.shared_types;


public final class LocationResponse {
    public final java.util.Optional<Coordinate> location;

    public LocationResponse(java.util.Optional<Coordinate> location) {
        java.util.Objects.requireNonNull(location, "location must not be null");
        this.location = location;
    }

    public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.increase_container_depth();
        TraitHelpers.serialize_option_Coordinate(location, serializer);
        serializer.decrease_container_depth();
    }

    public byte[] bincodeSerialize() throws com.novi.serde.SerializationError {
        com.novi.serde.Serializer serializer = new com.novi.bincode.BincodeSerializer();
        serialize(serializer);
        return serializer.get_bytes();
    }

    public static LocationResponse deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        deserializer.increase_container_depth();
        Builder builder = new Builder();
        builder.location = TraitHelpers.deserialize_option_Coordinate(deserializer);
        deserializer.decrease_container_depth();
        return builder.build();
    }

    public static LocationResponse bincodeDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
        if (input == null) {
             throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
        }
        com.novi.serde.Deserializer deserializer = new com.novi.bincode.BincodeDeserializer(input);
        LocationResponse value = deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.length) {
             throw new com.novi.serde.DeserializationError("Some input bytes were not read");
        }
        return value;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        LocationResponse other = (LocationResponse) obj;
        if (!java.util.Objects.equals(this.location, other.location)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + (this.location != null ? this.location.hashCode() : 0);
        return value;
    }

    public static final class Builder {
        public java.util.Optional<Coordinate> location;

        public LocationResponse build() {
            return new LocationResponse(
                location
            );
        }
    }
}