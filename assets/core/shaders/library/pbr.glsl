const float MAX_SPEC_LOD = 4.0;

vec3 f_schlick(const vec3 f0, const float vh) {
	return f0 + (1.0 - f0) * exp2((-5.55473 * vh - 6.98316) * vh);
}

float v_smithschlick(const float nl, const float nv, const float a) {
	return 1.0 / ((nl * (1.0 - a) + a) * (nv * (1.0 - a) + a));
}

float d_ggx(const float nh, const float a) {
	float a2 = a * a;
	float denom = pow(nh * nh * (a2 - 1.0) + 1.0, 2.0);
	return a2 * (1.0 / 3.1415926535) / denom;
}

vec3 specularBRDF(const vec3 f0, const float roughness, const float nl, const float nh, const float nv, const float vh) {
	float a = roughness * roughness;
	return d_ggx(nh, a) * clamp(v_smithschlick(nl, nv, a), 0.0, 1.0) * f_schlick(f0, vh) / 4.0;
}

vec3 lambertDiffuseBRDF(const vec3 albedo, const float nl) {
	return albedo * max(0.0, nl);
}

vec3 saturate(vec3 v) {
    return clamp(v, vec3(0.0), vec3(1.0));
}

float saturate(float v) {
    return clamp(v, 0.0, 1.0);
}