export async function test_dyn_import() {
    print("test_dyn_import");
    await import('typescript:src/b.tsx')
    let {React} = await import('src/react.js')
    let {b} = await import('src/b.tsx')
    let {a} = await import('../src/a.jsx')
    await import('../src/a.jsx')
    await import('react/umd/react.js')
    return "Done." + React + a + b
}

export function main(param) {
    return test_dyn_import()
}