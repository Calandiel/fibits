// Hilbert curve

pub fn xy2d(n : usize, x : usize, y : usize) -> usize
{
	let mut mx = x;
	let mut my = y;
	let mut d = 0;
	let mut s = n / 2;
	while s > 0 {
		let rx = match (mx & s) > 0 { true => 1, false => 0 };
		let ry = match (my & s) > 0 { true => 1, false => 0 };
		d += s * s * ((3 * rx) ^ ry);
		rotate(n, &mut mx, &mut my, rx, ry);
		s /= 2;
	}

	return d;
}

pub fn d2xy(n : usize, d : usize) -> (usize, usize)
{
	let mut t = d;
	let mut x = 0;
	let mut y = 0;
	let mut s = 1;
	while s < n {
		let rx = 1 & (t / 2);
		let ry = 1 & (t ^ rx);
		rotate(s, &mut x, &mut y, rx, ry);
		x += s * rx;
		y += s * ry;
		t /= 4;

		s *= 2;
	}

	(x, y)
}

fn rotate(n : usize, x : &mut usize, y : &mut usize, rx : usize, ry : usize)
{
	if ry == 0
	{
		if rx == 1
		{
			*x = n - 1 - *x;
			*y = n - 1 - *y;
		}
		//Swap x and y
		let t = *x;
		*x = *y;
		*y = t;
	}
}