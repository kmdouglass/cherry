/**
 * Tools for working with ray trace results from the ray tracer engine.
 * @module rays
 */

/**
 * A ray from the ray tracer engine.
 * @typedef Ray
 * @type {object}
 * @property {[number, number, number]} pos - The position of the ray.
 * @property {[number, number, number]} dir - The direction of the ray.
 */

/**
 * A bundle of rays traced through an optical system from the ray tracer.
 * @typedef RayBundle
 * @type {object}
 * @property {Array<Ray>} rays - The rays in the bundle.
 * @property {Array<number>} terminated - The surfaces indices where the corresponding ray terminated.
 * @property {Map<number, string>} reason_for_termination - The reason for termination of a given ray.
 * @property {number} num_surfaces - The number of surfaces in the optical system.
 */

/**
 * The results of tracing rays through an optical system for a single wavelength, field, and axis.
 * @typedef TraceResults
 * @type {object}
 * @property {number} wavelength_id - The wavelength ID of the ray.
 * @property {number} field_id - The field ID of the ray.
 * @property {String} axis - The axis used to compute the entrance pupil.
 * @property {RayBundle} ray_bundle - The ray bundle traced through the optical system.
 * @property {RayBundle} chief_ray - The chief ray traced through the optical system.
 */

/**
 * A collection of trace results from the ray tracer engine for multiple wavelengths, fields, and axes.
 * @typedef TraceResultsCollection
 * @type {object}
 * @property {Array<TraceResults>} results - The results of tracing rays through the optical system.
 */

/**
 * The description of an optical system returned by the ray tracer.
 * @typedef Description
 * @type {object}
 * @property {object} components_view - The components of the optical system.
 * @property {object} cutaway_view - The cutaway view of the optical system.
 * @property {object} paraxial_view - The paraxial view of the optical system.
 */
