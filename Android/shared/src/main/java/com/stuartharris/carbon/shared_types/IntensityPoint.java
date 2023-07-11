package com.stuartharris.carbon.shared_types;


public final class IntensityPoint {
    public final String date;
    public final Integer forecast;
    public final java.util.Optional<Integer> actual;

    public IntensityPoint(String date, Integer forecast, java.util.Optional<Integer> actual) {
        java.util.Objects.requireNonNull(date, "date must not be null");
        java.util.Objects.requireNonNull(forecast, "forecast must not be null");
        java.util.Objects.requireNonNull(actual, "actual must not be null");
        this.date = date;
        this.forecast = forecast;
        this.actual = actual;
    }

    public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.increase_container_depth();
        serializer.serialize_str(date);
        serializer.serialize_i32(forecast);
        TraitHelpers.serialize_option_i32(actual, serializer);
        serializer.decrease_container_depth();
    }

    public byte[] bincodeSerialize() throws com.novi.serde.SerializationError {
        com.novi.serde.Serializer serializer = new com.novi.bincode.BincodeSerializer();
        serialize(serializer);
        return serializer.get_bytes();
    }

    public static IntensityPoint deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        deserializer.increase_container_depth();
        Builder builder = new Builder();
        builder.date = deserializer.deserialize_str();
        builder.forecast = deserializer.deserialize_i32();
        builder.actual = TraitHelpers.deserialize_option_i32(deserializer);
        deserializer.decrease_container_depth();
        return builder.build();
    }

    public static IntensityPoint bincodeDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
        if (input == null) {
             throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
        }
        com.novi.serde.Deserializer deserializer = new com.novi.bincode.BincodeDeserializer(input);
        IntensityPoint value = deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.length) {
             throw new com.novi.serde.DeserializationError("Some input bytes were not read");
        }
        return value;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        IntensityPoint other = (IntensityPoint) obj;
        if (!java.util.Objects.equals(this.date, other.date)) { return false; }
        if (!java.util.Objects.equals(this.forecast, other.forecast)) { return false; }
        if (!java.util.Objects.equals(this.actual, other.actual)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + (this.date != null ? this.date.hashCode() : 0);
        value = 31 * value + (this.forecast != null ? this.forecast.hashCode() : 0);
        value = 31 * value + (this.actual != null ? this.actual.hashCode() : 0);
        return value;
    }

    public static final class Builder {
        public String date;
        public Integer forecast;
        public java.util.Optional<Integer> actual;

        public IntensityPoint build() {
            return new IntensityPoint(
                date,
                forecast,
                actual
            );
        }
    }
}
