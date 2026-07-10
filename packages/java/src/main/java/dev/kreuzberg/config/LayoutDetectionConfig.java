package dev.kreuzberg.config;

import java.util.HashMap;
import java.util.Map;

/**
 * Layout detection configuration for document layout analysis.
 *
 * @since 4.4.0
 */
public final class LayoutDetectionConfig {
    private final Double confidenceThreshold;
    private final Boolean applyHeuristics;
    private final String tableModel;
    private final AccelerationConfig acceleration;

    private LayoutDetectionConfig(Builder builder) {
        this.confidenceThreshold = builder.confidenceThreshold;
        this.applyHeuristics = builder.applyHeuristics;
        this.tableModel = builder.tableModel;
        this.acceleration = builder.acceleration;
    }

    public static Builder builder() {
        return new Builder();
    }

    public Double getConfidenceThreshold() {
        return confidenceThreshold;
    }

    public Boolean getApplyHeuristics() {
        return applyHeuristics;
    }

    public String getTableModel() {
        return tableModel;
    }

    /**
	 * Get the acceleration configuration.
	 *
	 * @return the acceleration configuration, or null if not set
	 * @since 4.5.0
	 */
    public AccelerationConfig getAcceleration() {
        return acceleration;
    }

    public Map<String, Object> toMap() {
        Map<String, Object> map = new HashMap<>();
        if (confidenceThreshold != null) {
            map.put("confidence_threshold", confidenceThreshold);
        }
        if (applyHeuristics != null) {
            map.put("apply_heuristics", applyHeuristics);
        }
        if (tableModel != null) {
            map.put("table_model", tableModel);
        }
        if (acceleration != null) {
            map.put("acceleration", acceleration.toMap());
        }
        return map;
    }

    public static final class Builder {
        private Double confidenceThreshold;
        private Boolean applyHeuristics = true;
        private String tableModel;
        private AccelerationConfig acceleration;

        private Builder() {
        }

        public Builder confidenceThreshold(Double confidenceThreshold) {
            this.confidenceThreshold = confidenceThreshold;
            return this;
        }

        public Builder applyHeuristics(Boolean applyHeuristics) {
            this.applyHeuristics = applyHeuristics;
            return this;
        }

        public Builder tableModel(String tableModel) {
            this.tableModel = tableModel;
            return this;
        }

        /**
		 * Set the acceleration configuration.
		 *
		 * @param acceleration
		 *            the acceleration configuration
		 * @return this builder for chaining
		 * @since 4.5.0
		 */
        public Builder acceleration(AccelerationConfig acceleration) {
            this.acceleration = acceleration;
            return this;
        }

        public LayoutDetectionConfig build() {
            return new LayoutDetectionConfig(this);
        }
    }

    static LayoutDetectionConfig fromMap(Map<String, Object> map) {
        if (map == null) {
            return null;
        }
        Builder builder = builder();
        Object confidenceThresholdValue = map.get("confidence_threshold");
        if (confidenceThresholdValue instanceof Number) {
            builder.confidenceThreshold(((Number) confidenceThresholdValue).doubleValue());
        }
        Object applyHeuristicsValue = map.get("apply_heuristics");
        if (applyHeuristicsValue instanceof Boolean) {
            builder.applyHeuristics((Boolean) applyHeuristicsValue);
        }
        if (map.get("table_model") instanceof String) {
            builder.tableModel((String) map.get("table_model"));
        }
        Map<String, Object> accelerationMap = asMap(map.get("acceleration"));
        if (accelerationMap != null) {
            builder.acceleration(AccelerationConfig.fromMap(accelerationMap));
        }
        return builder.build();
    }

    @SuppressWarnings({"unchecked", "PMD.ReturnEmptyCollectionRatherThanNull"})
    private static Map<String, Object> asMap(Object value) {
        if (value instanceof Map) {
            return (Map<String, Object>) value;
        }
        return null;
    }
}
