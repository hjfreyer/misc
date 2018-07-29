import sympy

from sympy.matrices import *

#N = vector.ReferenceFrame('N')

tf = sympy.symbols('tf')

g_const, target = sympy.symbols('g_const target')

T = Matrix(3, 1, sympy.symbols('T0:3'))
G = Matrix([0, 0, - g_const])

V_0 = Matrix(3, 1, sympy.symbols('V0:3'))
P_0 = Matrix(3, 1, sympy.symbols('P0:3'))

V_f = V_0 + (T+G) * tf

P_f = P_0 + V_0 * tf + (T+G)/2 * tf**2

eqns = [
    V_f,
    P_f[2, 0] - target
]

"""

G = -N.y * g_const
T = t_max *(N.y * sympy.cos(theta) + N.x * sympy.sin(theta))

v_0_x, v_0_y =  sympy.symbols('v_0_x v_0_y')
V_0 = N.x * v_0_x + N.y * v_0_y

V_tb = V_0 + G * tb
V_tf = V_tb + (G + T) * (tf - tb)

p_0_y = sympy.symbols('p_0_y')
P_0 = N.y * p_0_y

P_tb = P_0 + V_0 * tb + (G * tb**2) / 2
P_tf = P_tb + V_tf * (tf-tb) + (T + G) * (tb**2) / 2

"""
