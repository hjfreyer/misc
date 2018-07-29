
import collections

import tensorflow as tf

Maybe = collections.namedtuple('Maybe', 'has value')
Phase = collections.namedtuple('Phase', 'initializer next_phase outs')

def MaybeSo(v):
    return Maybe(has=tf.constant(True),
                 value=v)

def MaybeNot():
    return Maybe(has=tf.constant(False),
                 value=tf.constant(0.0, tf.double))

def _LiquidFuelThatWillGo(c, v):
    next_decouple = v.control.current_stage - 1
    res = v.resources_in_decouple_stage(next_decouple, cumulative=False)
    return res.amount('LiquidFuel') + res.amount('SolidFuel')

def _NodeTime(c, v):
    if not v.control.nodes:
        return 0.0
    return v.control.nodes[0].ut

def _NodeVector(c, v):
    if not v.control.nodes:
        return (0.0, 0.0, 0.0)
    return v.control.nodes[0].remaining_burn_vector(v.auto_pilot.reference_frame)

def _SurfVel(c, v):
    ref_frame = c.space_center.ReferenceFrame.create_hybrid(
        position=v.orbit.body.reference_frame,
        rotation=v.surface_reference_frame)
    return v.flight(ref_frame).velocity
def _NoRotateSurfVel(c, v):
    ref_frame = c.space_center.ReferenceFrame.create_hybrid(
        position=v.orbit.body.non_rotating_reference_frame,
        rotation=v.surface_reference_frame)
    return v.flight(ref_frame).velocity

def _AutopilotError(c, v):
    try:
        return v.auto_pilot.error
    except:
        return 0.0

_Inputs = [
    ('t', lambda c, v: c.space_center.ut),
    ('alt', lambda c, v: v.flight(v.orbit.body.reference_frame).surface_altitude),
    ('available_thrust', lambda c, v: v.available_thrust),
    ('thrust', lambda c, v: v.thrust),
    ('mass', lambda c, v: v.mass),
    ('air_speed', lambda c, v: v.flight(v.orbit.body.reference_frame).true_air_speed),
    ('surface_velocity', _SurfVel),
    ('no_rotate_surface_velocity', _NoRotateSurfVel),
    ('autopilot_error', _AutopilotError),
    ('speed', lambda c, v: v.orbit.speed),
    ('term_vel', lambda c, v: v.flight(v.orbit.body.reference_frame).terminal_velocity),
    ('mu', lambda c, v: v.orbit.body.gravitational_parameter),
    ('orbit_a', lambda c, v: v.orbit.semi_major_axis),
    ('radius', lambda c, v: v.orbit.radius),
    ('apo', lambda c, v: v.orbit.apoapsis),
    ('time_to_apo', lambda c, v: v.orbit.time_to_apoapsis),
    ('stage', lambda c, v: v.control.current_stage),
    ('solid_fuel', lambda c, v: v.resources.amount('SolidFuel')),
    ('liquid_fuel_to_go', _LiquidFuelThatWillGo),
    ('periapsis', lambda c, v: v.orbit.periapsis),
    ('prograde', lambda c, v: v.flight(v.auto_pilot.reference_frame).prograde),
    ('retrograde', lambda c, v: v.flight(v.auto_pilot.reference_frame).retrograde),
    ('has_node', lambda c, v: len(v.control.nodes) > 0),
    ('node_vector', _NodeVector),
    ('node_time', _NodeTime),
    ('surface_gravity', lambda c, v: v.orbit.body.surface_gravity),
]

Inputs = collections.namedtuple('Inputs', [k for k, v in _Inputs])

InputPlaceholders = Inputs._make(tf.placeholder(tf.double, name=k) for k, _ in _Inputs)

def FeedDict(conn):
    v = conn.space_center.active_vessel
    values = [val(conn, v) for _, val in _Inputs]
    return dict(zip(InputPlaceholders, values))

_Outputs = [
    ('throttle', lambda c, v: (setattr, v.control, 'throttle')),
    ('stage', lambda c, v: (lambda _: v.control.activate_next_stage(), )),
    ('target_direction', lambda c, v: (setattr, v.auto_pilot, 'target_direction')),
    ('warp', lambda c, v: (setattr, c.space_center, 'rails_warp_factor')),
#    ('rails_warp_factor', lambda c: (setattr, c.space_center, 'rails_warp_factor')),
]

OutputsT = collections.namedtuple('OutputsT',
                                  [k for k, _ in _Outputs])

def Outputs(**kwargs):
    for k, _ in _Outputs:
        if k not in kwargs:
            kwargs[k] = MaybeNot()
    return OutputsT(**kwargs)

def HandleOutputs(conn, outputs):
    v = conn.space_center.active_vessel
    for val, (name, out_builder) in zip(outputs, _Outputs):
        if val.has:
            outer = out_builder(conn, v)
            func, args = outer[0], outer[1:] + (val.value,)
            func(*args)
