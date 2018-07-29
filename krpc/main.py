import krpc
import time
import tensorflow as tf
import collections
import math

import kio

Maybe = collections.namedtuple('Maybe', 'has value')
Phase = collections.namedtuple('Phase', 'initializer next_phase outs')

def MaybeSo(v):
    return Maybe(has=tf.constant(True),
                 value=v)

def MaybeNot():
    return Maybe(has=tf.constant(False),
                 value=tf.constant(0.0, tf.double))

ATMO_CLEAR = 680000

_PhaseNames = ['GRAVITY_TURN', 'COAST', 'ENTER_ORBIT', 'CIRCULARIZE',
               'RENDEZVOUS', 'KILL_HORIZONTAL', 'HOVER', 'LANDING']
_PhaseNamesTuple = collections.namedtuple('PhaseNames', _PhaseNames)
PhaseNames = _PhaseNamesTuple(*_PhaseNames)


_DIFF_MEMO = {}
def Diff(tensor):
    if tensor not in _DIFF_MEMO:
        initialized = tf.Variable(False)
        prev_val = tf.Variable(0, dtype=tensor.dtype)
        prev_val_shadow = tf.Variable(0, dtype=tensor.dtype)

        tf.add_to_collection('COMPUTE_SHADOWS',
                             tf.assign(prev_val_shadow, tensor))
        tf.add_to_collection('ASSIGN_VALUES', tf.assign(initialized, True))
        tf.add_to_collection('ASSIGN_VALUES',
                             tf.assign(prev_val, prev_val_shadow))

        _DIFF_MEMO[tensor] = tf.cond(initialized,
                                     lambda: tensor - prev_val,
                                     lambda: tf.constant(0, dtype=tensor.dtype))
    return _DIFF_MEMO[tensor]

def ClippedSum(integrand, min_val, max_val):
    sum = tf.Variable(0, dtype=integrand.dtype)
    shadow_sum = tf.Variable(0, dtype=integrand.dtype)

    next_sum = sum + integrand
    next_sum = tf.minimum(next_sum, max_val)
    next_sum = tf.maximum(next_sum, min_val)

    tf.add_to_collection('COMPUTE_SHADOWS',
                         tf.assign(shadow_sum, next_sum))
    tf.add_to_collection('ASSIGN_VALUES',
                         tf.assign(sum, shadow_sum))

    resetter = lambda x: tf.group(tf.assign(shadow_sum, x), tf.assign(sum, x))

    return sum, resetter

def dbl(x):
    return tf.constant(x, tf.double)

def PIController(time, err, k_p, t_i, cv_min, cv_max):
    derr = Diff(err)
    dt = Diff(time)

    dcv = k_p * derr + k_p * err * dt / t_i
    dcv = tf.cond(tf.is_nan(dcv), lambda: dbl(0), lambda: dcv)
    #dcv = tf.Print(dcv, [err])
    return ClippedSum(dcv, cv_min, cv_max)[0]

def EnginePIController(inp, err):
    available_acc = inp.available_thrust / inp.mass

    k_p = 2.0 / available_acc
    t_i = 8.0

    return PIController(inp.t, err, k_p, t_i, dbl(0), dbl(1))

DUE_EAST = tf.convert_to_tensor((0.0, 0.0, 1.0), dtype=tf.double)

class GravityTurn(object):
    def Name(self): return PhaseNames.GRAVITY_TURN

    def Start(self, conn):
        return tf.no_op()

    def End(self, conn):
        conn.space_center.active_vessel.control.throttle = 0
        return tf.no_op()

    def Graph(self, inp):
        grav_turn_angle = 90.0 * (1 - (inp.alt - 2000.0) / 50000.0)
        grav_turn_angle = tf.minimum(tf.constant(90.0, tf.double), grav_turn_angle)
        grav_turn_angle = tf.maximum(tf.constant(0.0, tf.double), grav_turn_angle)

        should_stage = tf.equal(inp.liquid_fuel_to_go, 0.0)

        dt = Diff(inp.t)
        err = inp.term_vel - inp.air_speed

        throttle = EnginePIController(inp, err)

        pitch = grav_turn_angle * math.pi / 180.0

        targetdir = tf.convert_to_tensor((tf.sin(pitch), 0.0, tf.cos(pitch)))

        outs = kio.Outputs(
            throttle=MaybeSo(throttle),
            target_direction=MaybeSo(targetdir),
            stage=Maybe(has=should_stage, value=tf.constant(0, tf.double)),
        )

        next_phase = tf.cond(inp.apo > ATMO_CLEAR + 10000,
                             lambda: PhaseNames.COAST,
                             lambda: PhaseNames.GRAVITY_TURN)
        return next_phase, outs


class Coast(object):
    def Name(self): return PhaseNames.COAST

    def Start(self, conn):
        return tf.no_op()

    def End(self, conn):
        return tf.no_op()

    def Graph(self, inp):
        outs = kio.Outputs(
            throttle=MaybeSo(tf.constant(0.0, tf.double)),
            target_direction=MaybeSo(DUE_EAST),
            stage=MaybeNot(),
        )

        next_phase = tf.cond(inp.alt > 70000,
                       lambda: PhaseNames.ENTER_ORBIT,
                       lambda: PhaseNames.COAST)

        return next_phase, outs

def ExecNode(inp, this_phase, next_phase):
    delta_v = tf.norm(inp.node_vector)
    available_acc = inp.available_thrust / inp.mass

    throttle = 1.0 * delta_v / available_acc

    burn_duration = delta_v / available_acc

    throttle = tf.cond(inp.node_time - burn_duration / 2 < inp.t,
                       lambda: throttle,
                       lambda: dbl(0))
    throttle = tf.cond(inp.autopilot_error < 3,
                       lambda: throttle,
                       lambda: dbl(0))
    should_stage = inp.liquid_fuel_to_go < 0.1

    outs = kio.Outputs(
        throttle=MaybeSo(throttle),
        target_direction=MaybeSo(inp.node_vector),
        stage=Maybe(has=should_stage, value=tf.constant(0, tf.double)),
    )

    next_phase = tf.cond(delta_v < 0.1,
                         lambda: next_phase,
                         lambda: this_phase)

    return next_phase, outs


def CircularizeStart(conn):
    v = conn.space_center.active_vessel

    v.control.throttle = 0

    mu = v.orbit.body.gravitational_parameter

    energy = (-mu / v.orbit.semi_major_axis / 2)
    target_energy = -mu / v.orbit.apoapsis / 2

    speed_at_apo = math.sqrt(2*(energy + mu / v.orbit.apoapsis))
    target_speed_at_apo = math.sqrt(mu / v.orbit.apoapsis)
    delta_v = target_speed_at_apo - speed_at_apo

    available_acc = v.available_thrust / v.mass

    burn_duration = delta_v / available_acc
    burn_start = conn.space_center.ut + v.orbit.time_to_apoapsis - burn_duration / 2

    v.control.remove_nodes()
    v.control.add_node(conn.space_center.ut + v.orbit.time_to_apoapsis,
                       prograde=delta_v)

    conn.space_center.warp_to(burn_start - 20)

    return tf.no_op()


class EnterOrbit(object):
    def Name(self): return PhaseNames.ENTER_ORBIT

    def Start(self, conn):
        return CircularizeStart(conn)

    def End(self, conn):
        conn.space_center.active_vessel.control.throttle = 0
        return tf.no_op()

    def Graph(self, inp):
        return ExecNode(inp, PhaseNames.ENTER_ORBIT, PhaseNames.CIRCULARIZE)


class Circularize(object):
    def Name(self): return PhaseNames.CIRCULARIZE

    def Start(self, conn):
        return CircularizeStart(conn)

    def End(self, conn):
        conn.space_center.active_vessel.control.throttle = 0
        return tf.no_op()

    def Graph(self, inp):
        return ExecNode(inp, PhaseNames.CIRCULARIZE, PhaseNames.RENDEZVOUS)


class Rendezvous(object):
    def Name(self): return PhaseNames.RENDEZVOUS

    def Start(self, conn):
        v = conn.space_center.active_vessel
        mun = conn.space_center.bodies['Mun']
        mu = v.orbit.body.gravitational_parameter

        falpha = lambda o: (
            o.mean_anomaly + o.argument_of_periapsis
            + o.longitude_of_ascending_node)

        tau = 2 * math.pi
        alpha_0 = falpha(mun.orbit) - falpha(v.orbit)
        omega = tau / mun.orbit.period
        omega_rel = tau / mun.orbit.period - tau / v.orbit.period
        transfer_a = (v.orbit.radius + mun.orbit.radius) / 2
        transfer_duration = math.sqrt(4 * math.pi**2 * transfer_a**3 / mu) / 2

        phase_angle = math.pi - transfer_duration * omega
        relative_period = abs(tau / omega_rel)

        t = (phase_angle  / omega_rel - alpha_0 / omega_rel) % relative_period

        time_to_burn = conn.space_center.ut + t

        energy = -mu / v.orbit.radius / 2
        target_energy = -mu / transfer_a / 2

        target_speed = math.sqrt(2*(target_energy + mu / v.orbit.radius))
        delta_v = target_speed - v.orbit.speed

        v.control.remove_nodes()
        v.control.add_node(time_to_burn, prograde=delta_v)

        conn.space_center.warp_to(time_to_burn - 30)

        return tf.no_op()

    def End(self, conn):
        v = conn.space_center.active_vessel
        v.control.remove_nodes()
        v.control.throttle = 0
        conn.space_center.warp_to(conn.space_center.ut + v.orbit.time_to_soi_change + 10)
        return tf.no_op()

    def Graph(self, inp):
        #return tf.constant(PHASE_RENDEZVOUS), kio.Outputs(
    #        throttle=MaybeSo(dbl(0)),
    #        target_direction=MaybeSo(inp.node_vector),
    #        stage=MaybeNot(),
    #    )
        return ExecNode(inp, PhaseNames.RENDEZVOUS, PhaseNames.HOVER)

class KillHorizontal(object):
    def Name(self): return PhaseNames.KILL_HORIZONTAL

    def Start(self, conn):
        return tf.no_op()
    def End(self, conn):
        return tf.no_op()

    def Graph(self, inp):
        v = inp.no_rotate_surface_velocity
        vhoriz = tf.sqrt(v[1]**2 + v[2]**2)
        target_dir = tf.convert_to_tensor((0, -v[1], -v[2]))
        available_acc = inp.available_thrust / inp.mass

        throttle = 2 * vhoriz / available_acc
        throttle = tf.cond(
            inp.autopilot_error < 1,
            lambda: throttle,
            lambda: dbl(0.0),
        )

        next_phase = tf.cond(
            vhoriz < 0.1,
            lambda: tf.constant(PhaseNames.HOVER),
            lambda: tf.constant(PhaseNames.KILL_HORIZONTAL)
        )

        return next_phase, kio.Outputs(
               throttle=MaybeSo(throttle),
               target_direction=MaybeSo(target_dir),
               stage=MaybeNot(),
        )

def GetErrorForAngle(v_vert, v_horiz, acc_engine, acc_gravity, theta):
    acc_vert = math.cos(theta) * acc_engine - acc_gravity
    acc_horiz = math.sin(theta) * acc_engine

    time_to_horiz_stop = v_horiz / acc_horiz


class Hover(object):
    def Name(self): return PhaseNames.HOVER

    def __init__(self):
        pass
    def Start(self, conn):
        v = conn.space_center.active_vessel
        if v.control.current_stage != 0:
            v.control.activate_next_stage()

        v.auto_pilot.target_direction = v.flight(v.auto_pilot.reference_frame).retrograde
        v.auto_pilot.wait()
        return self.resetter(0)

    def End(self, conn):
        return tf.no_op()

    def Graph(self, inp):
        TARGET_ALT = 10
        MAXIMUM_GROUND_GRADE = 5.0  # 5 vertical meters per horizontal meter.
        WORST_CASE_LOOP_DELAY = 0.2
        MUN_RADIUS = 200000
        MUN_MAX_HEIGHT = 7500

        current_gravity = inp.mu / (inp.radius ** 2)
        gravity = (current_gravity + inp.surface_gravity) / 2
        available_acc = inp.available_thrust / inp.mass

        v = inp.surface_velocity
        worst_case_vertical_velocity = (
            -v[0]  # Current velocity (down).
            # Rate at which the ground might be coming up.
            + tf.sqrt(v[1]**2 + v[2]**2) * MAXIMUM_GROUND_GRADE
            # Extra velocity due to acceleration.
            + gravity * WORST_CASE_LOOP_DELAY)
        worst_case_vertical_error = worst_case_vertical_velocity * WORST_CASE_LOOP_DELAY

        # Check if the altimeter is just reporting distance to sea level.
        alt_is_msl = tf.abs(inp.radius - MUN_RADIUS - inp.alt) < 0.1
        worst_case_vertical_error = tf.cond(
            alt_is_msl,
            lambda: worst_case_vertical_error + MUN_MAX_HEIGHT,
            lambda: worst_case_vertical_error)

        # Vectors are relative to surface_velocity_reference_frame.
        dist = (inp.alt - TARGET_ALT - worst_case_vertical_error)
        tx = (dist * gravity + v[0]**2 / 2) / dist
        ty = v[1] * v[0] / (2*dist)
        tz = v[2] * v[0] / (2*dist)

        thrust_acc = tf.convert_to_tensor((tx, ty, tz))

        retrograde = thrust_acc

        ideal_suicide_burn_distance = v[0]**2 / (2*(available_acc - gravity))

        throttle1 = tf.norm(thrust_acc) / available_acc
        warp_dist = dist - ideal_suicide_burn_distance * 2
        throttle1 = tf.Print(throttle1, [throttle1, inp.alt, warp_dist])
        throttle = tf.cond(0.95 < throttle1,
                           lambda: throttle1,
                           lambda: dbl(0))

        next_phase = tf.cond(
            v[0] > 0.0,
            lambda: tf.constant(PhaseNames.LANDING),
            lambda: tf.constant(PhaseNames.HOVER)
        )

        w, self.resetter = WarpDown(warp_dist)

        return next_phase, kio.Outputs(
               throttle=MaybeSo(throttle),
               target_direction=MaybeSo(retrograde),
               stage=MaybeNot(),
               warp=MaybeSo(tf.floor(w)),
        )

class Landing(object):
    def Name(self): return PhaseNames.LANDING

    def Start(self, conn):
        return tf.no_op()

    def End(self, conn):
        return tf.no_op()

    def Graph(self, inp):
        v = inp.surface_velocity

        err = tf.maximum(-1 - v[0], tf.sqrt(v[1]**2 + v[2]**2))

        upish = tf.convert_to_tensor((1, -v[1], -v[2]))

        throttle = EnginePIController(inp, err)
        throttle = tf.cond(inp.autopilot_error < 3,
                            lambda: throttle,
                            lambda: dbl(0))
        return tf.constant(PhaseNames.LANDING), kio.Outputs(
               throttle=MaybeSo(throttle),
               target_direction=MaybeSo(upish),
               stage=MaybeNot(),
           )

def WarpDown(value):
    dv = Diff(value)

    etz = - value / dv

    delta = tf.case([
        (etz < 10, lambda: dbl(-0.5)),
        (etz > 100, lambda: dbl(0.1))
    ],default=lambda: dbl(0))

    warp, resetter = ClippedSum(delta, dbl(0), dbl(7))
    warp = tf.cond(value < 0, lambda: dbl(0), lambda: warp)
    return warp, resetter

def main(argv):
    conn = krpc.connect(name='Hello World')
    vessel = conn.space_center.active_vessel

    phase = PhaseNames.GRAVITY_TURN
    if argv:
        phase = argv[0]
    with tf.Session() as sess:
        inp = kio.InputPlaceholders

        last_phase = None
        phase_list = [
            GravityTurn(),
            Coast(),
            EnterOrbit(),
            Circularize(),
            Rendezvous(),
            KillHorizontal(),
            Hover(),
            Landing(),
        ]

        phases = dict((p.Name(), p) for p in phase_list)

        graphs = dict((name, p.Graph(inp)) for name, p in phases.iteritems())

        compute_shadows_op = tf.group(tf.get_collection('COMPUTE_SHADOWS'))
        assign_values_op = tf.group(tf.get_collection('ASSIGN_VALUES'))

        vessel.auto_pilot.engage()
        sess.run(tf.global_variables_initializer())
        while True:
            if last_phase != phase:
                if last_phase:
                    sess.run(phases[last_phase].End(conn))
                print 'PHASE %s' % phase
                sess.run(phases[phase].Start(conn))
                last_phase = phase

            fd = kio.FeedDict(conn)

            g_next_phase, g_outs = graphs[phase]
            phase, outs, _ = sess.run(
                (g_next_phase, g_outs,
                 compute_shadows_op),
                feed_dict=fd)
            kio.HandleOutputs(conn, outs)

            sess.run(tf.get_collection('ASSIGN_VALUES'))
            time.sleep(0.1)

import sys
main(sys.argv[1:])
